use crate::auth::AuthUser;
use crate::db::connect;
use crate::models::{ReadingEntry, ReadingGoal, ReadingGoalTimeframe, ReadingGoalType};
use crate::schema::reading_entries::dsl::reading_entries;
use crate::schema::reading_goals::dsl::reading_goals;
use crate::schema::readings::dsl::readings;
use crate::{schema, ErrorResponse};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use chrono::{Datelike, NaiveDate, TimeDelta};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/goals/create", post(create_goal))
        .route("/api/goals/list", post(list_goals))
        .route("/api/goals/delete", post(delete_goal))
}

fn get_period(timeframe: &ReadingGoalTimeframe) -> (NaiveDate, NaiveDate) {
    let today = chrono::Utc::now().date_naive();
    match timeframe {
        ReadingGoalTimeframe::Year => (
            NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(today.year(), 12, 31).unwrap(),
        ),
        ReadingGoalTimeframe::Month => {
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            let end = if today.month() == 12 {
                NaiveDate::from_ymd_opt(today.year(), 12, 31).unwrap()
            } else {
                NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
                    .unwrap()
                    .pred_opt()
                    .unwrap()
            };
            (start, end)
        }
        ReadingGoalTimeframe::Week => {
            let weekday = today.weekday().num_days_from_monday();
            let start = today - TimeDelta::days(weekday as i64);
            let end = start + TimeDelta::days(6);
            (start, end)
        }
    }
}

fn calculate_books_progress(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> i64 {
    let count: i64 = match readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.is_not_null())
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .count()
        .get_result(connection)
    {
        Ok(c) => c,
        Err(_) => return 0,
    };
    count
}

fn calculate_pages_progress(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> i64 {
    let user_readings: Vec<Uuid> = match readings
        .select(schema::readings::dsl::id)
        .filter(schema::readings::dsl::user.eq(user_id))
        .load::<Uuid>(connection)
    {
        Ok(ids) => ids,
        Err(_) => return 0,
    };

    let mut total_pages: i64 = 0;

    for reading_id in &user_readings {
        let entries: Vec<ReadingEntry> = match reading_entries
            .filter(schema::reading_entries::dsl::reading.eq(reading_id))
            .filter(schema::reading_entries::dsl::user.eq(user_id))
            .order(schema::reading_entries::dsl::read_at.asc())
            .load(connection)
        {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut prev_progress: i64 = 0;

        for entry in &entries {
            if entry.read_at < period_start {
                prev_progress = entry.progress as i64;
            } else {
                break;
            }
        }

        for entry in &entries {
            if entry.read_at >= period_start && entry.read_at <= period_end {
                let current = entry.progress as i64;
                if current > prev_progress {
                    total_pages += current - prev_progress;
                }
                prev_progress = current;
            } else if entry.read_at > period_end {
                break;
            }
        }
    }

    total_pages
}

/// Request type for creating a reading goal.
#[derive(Debug, Deserialize)]
pub struct CreateGoalRequest {
    pub goal_type: String,
    pub timeframe: String,
    pub target: i32,
}

/// Creates a new reading goal for the authenticated user.
///
/// This route accepts a JSON payload with the following structure:
/// - `goal_type`: The type of goal, either `"books"` or `"pages"`.
/// - `timeframe`: The timeframe for the goal, one of `"year"`, `"month"`, or `"week"`.
/// - `target`: The target number of books or pages to read within the timeframe.
///
/// Only one goal per type and timeframe combination is allowed.
pub(crate) async fn create_goal(
    auth: AuthUser,
    Json(payload): Json<CreateGoalRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let goal_type = match payload.goal_type.as_str() {
        "books" => ReadingGoalType::Books,
        "pages" => ReadingGoalType::Pages,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid goal type. Must be 'books' or 'pages'.".to_string()
                })),
            )
        }
    };

    let timeframe = match payload.timeframe.as_str() {
        "year" => ReadingGoalTimeframe::Year,
        "month" => ReadingGoalTimeframe::Month,
        "week" => ReadingGoalTimeframe::Week,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid timeframe. Must be 'year', 'month', or 'week'.".to_string()
                })),
            )
        }
    };

    if payload.target <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse {
                error: "Target must be greater than 0.".to_string()
            })),
        );
    }

    let existing: i64 = match reading_goals
        .filter(schema::reading_goals::dsl::user_id.eq(auth.0))
        .filter(schema::reading_goals::dsl::goal_type.eq(&goal_type))
        .filter(schema::reading_goals::dsl::timeframe.eq(&timeframe))
        .count()
        .get_result(connection)
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error checking existing goals: {}", e)
                })),
            )
        }
    };

    if existing > 0 {
        return (
            StatusCode::CONFLICT,
            Json(json!(ErrorResponse {
                error: "A goal with this type and timeframe already exists.".to_string()
            })),
        );
    }

    let now = chrono::Utc::now().naive_utc();
    let new_goal = ReadingGoal {
        id: Uuid::new_v4(),
        user_id: auth.0,
        goal_type,
        timeframe,
        target: payload.target,
        created_at: now,
        updated_at: now,
    };

    match diesel::insert_into(schema::reading_goals::dsl::reading_goals)
        .values(&new_goal)
        .execute(connection)
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "id": new_goal.id.to_string(),
                "goal_type": new_goal.goal_type.to_string(),
                "timeframe": new_goal.timeframe.to_string(),
                "target": new_goal.target,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error creating goal: {}", e)
            })),
        ),
    }
}

/// Lists all reading goals for the authenticated user.
///
/// Returns each goal along with its current progress and percentage completion
/// calculated based on the goal's timeframe period.
pub(crate) async fn list_goals(auth: AuthUser) -> impl IntoResponse {
    let connection = &mut connect();

    let db_goals: Vec<ReadingGoal> = match reading_goals
        .filter(schema::reading_goals::dsl::user_id.eq(auth.0))
        .order(schema::reading_goals::dsl::created_at.desc())
        .load(connection)
    {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error loading goals: {}", e)
                })),
            )
        }
    };

    let mut json_goals = Vec::new();

    for goal in db_goals {
        let (period_start, period_end) = get_period(&goal.timeframe);

        let progress = match goal.goal_type {
            ReadingGoalType::Books => {
                calculate_books_progress(connection, auth.0, period_start, period_end)
            }
            ReadingGoalType::Pages => {
                calculate_pages_progress(connection, auth.0, period_start, period_end)
            }
        };

        let percentage = if goal.target > 0 {
            ((progress as f64 / goal.target as f64) * 100.0).round() as i64
        } else {
            0
        };

        json_goals.push(json!({
            "id": goal.id.to_string(),
            "goal_type": goal.goal_type.to_string(),
            "timeframe": goal.timeframe.to_string(),
            "target": goal.target,
            "progress": progress,
            "percentage": percentage,
            "period_start": period_start.to_string(),
            "period_end": period_end.to_string(),
        }));
    }

    (StatusCode::OK, Json(json!({ "goals": json_goals })))
}

/// Request type for deleting a reading goal.
#[derive(Debug, Deserialize)]
pub struct DeleteGoalRequest {
    pub goal_id: String,
}

/// Deletes a reading goal for the authenticated user.
///
/// This route accepts a JSON payload with the following structure:
/// - `goal_id`: The UUID of the goal to delete.
///
/// Returns 403 if the goal belongs to a different user.
pub(crate) async fn delete_goal(
    auth: AuthUser,
    Json(payload): Json<DeleteGoalRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let goal_id = match Uuid::parse_str(&payload.goal_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid goal ID.".to_string()
                })),
            )
        }
    };

    let goal: ReadingGoal = match reading_goals
        .filter(schema::reading_goals::dsl::id.eq(goal_id))
        .first(connection)
    {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!(ErrorResponse {
                    error: "Goal not found.".to_string()
                })),
            )
        }
    };

    if goal.user_id != auth.0 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!(ErrorResponse {
                error: "Access denied.".to_string()
            })),
        );
    }

    match diesel::delete(reading_goals.filter(schema::reading_goals::dsl::id.eq(goal_id)))
        .execute(connection)
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Goal deleted successfully." })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error deleting goal: {}", e)
            })),
        ),
    }
}
