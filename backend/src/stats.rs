use crate::auth::AuthUser;
use crate::db::connect;
use crate::goals::{calculate_books_progress, calculate_pages_progress};
use crate::schema::books::dsl::books;
use crate::schema::reading_entries::dsl::reading_entries;
use crate::schema::readings::dsl::readings;
use crate::{schema, ErrorResponse};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use chrono::{Datelike, NaiveDate, TimeDelta};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub(crate) fn register_routes(router: Router) -> Router {
    router.route("/api/stats/overview", post(overview))
}

fn year_period(year: i32) -> (NaiveDate, NaiveDate) {
    (
        NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
    )
}

fn month_period(year: i32, month: u32) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let end = if month == 12 {
        NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    };
    (start, end)
}

/// Counts the distinct days with logged reading entries within the period.
fn calculate_reading_days(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> i64 {
    match reading_entries
        .filter(schema::reading_entries::dsl::user.eq(user_id))
        .filter(schema::reading_entries::dsl::read_at.ge(period_start))
        .filter(schema::reading_entries::dsl::read_at.le(period_end))
        .select(schema::reading_entries::dsl::read_at)
        .distinct()
        .count()
        .get_result(connection)
    {
        Ok(c) => c,
        Err(_) => 0,
    }
}

/// Counts the consecutive days with logged reading entries as of the reference
/// date (a streak stays alive until a full day is missed). For past periods
/// this is the streak that was active at the end of the period.
fn calculate_reading_streak(connection: &mut PgConnection, user_id: Uuid, reference: NaiveDate) -> i64 {
    let dates: Vec<NaiveDate> = match reading_entries
        .filter(schema::reading_entries::dsl::user.eq(user_id))
        .select(schema::reading_entries::dsl::read_at)
        .distinct()
        .load(connection)
    {
        Ok(d) => d,
        Err(_) => return 0,
    };

    let days: HashSet<NaiveDate> = dates.into_iter().collect();
    let day_before = reference - TimeDelta::days(1);

    let mut cursor = if days.contains(&reference) {
        reference
    } else if days.contains(&day_before) {
        day_before
    } else {
        return 0;
    };

    let mut streak = 0;
    while days.contains(&cursor) {
        streak += 1;
        cursor -= TimeDelta::days(1);
    }
    streak
}

/// Averages the ratings of distinct books finished within the period.
/// Returns `None` when no finished book has a rating.
fn calculate_average_rating(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Option<f64> {
    let rows: Vec<(Uuid, Option<i16>)> = match readings
        .inner_join(books)
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.is_not_null())
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .select((schema::readings::dsl::book, schema::books::dsl::rating))
        .load(connection)
    {
        Ok(r) => r,
        Err(_) => return None,
    };

    // Re-reads of the same book only count once.
    let ratings: HashMap<Uuid, i16> = rows
        .into_iter()
        .filter_map(|(book, rating)| rating.map(|r| (book, r)))
        .collect();

    if ratings.is_empty() {
        return None;
    }

    let sum: i64 = ratings.values().map(|&r| r as i64).sum();
    Some(sum as f64 / ratings.len() as f64)
}

/// Request type for the stats overview.
#[derive(Debug, Deserialize)]
pub struct OverviewRequest {
    pub mode: String,
    pub year: i32,
    pub month: Option<u32>,
}

/// Returns key reading facts of a year or month for the authenticated user.
///
/// This route accepts a JSON payload with the following structure:
/// - `mode`: Either `"year"` or `"month"`.
/// - `year`: The year to report on (must not be in the future).
/// - `month`: The month (1-12) to report on, required for mode `"month"`.
///
/// The response contains the requested period and the following stats:
/// - `books_read`: readings finished in the period
/// - `pages_read`: pages logged in the period
/// - `books_added`: books added to the library in the period
/// - `reading_days`: distinct days with logged progress in the period
/// - `reading_streak_days`: streak of consecutive reading days at the end of
///   the period (past periods report the streak active back then)
/// - `average_rating`: average rating of books finished in the period
///   (may be null)
pub(crate) async fn overview(
    auth: AuthUser,
    Json(payload): Json<OverviewRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let today = chrono::Utc::now().date_naive();
    let current_year = today.year();

    let bad_request = |message: &str| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse {
                error: message.to_string()
            })),
        )
    };

    if payload.year < 1970 || payload.year > current_year {
        return bad_request("Invalid year.");
    }

    let (period_start, period_end) = match payload.mode.as_str() {
        "year" => year_period(payload.year),
        "month" => match payload.month {
            Some(month) if (1..=12).contains(&month) => {
                let period = month_period(payload.year, month);
                if period.0 > today {
                    return bad_request("Month must not be in the future.");
                }
                period
            }
            _ => return bad_request("Invalid month. Must be between 1 and 12."),
        },
        _ => return bad_request("Invalid mode. Must be 'year' or 'month'."),
    };

    let books_read = calculate_books_progress(connection, auth.0, period_start, period_end);
    let pages_read = calculate_pages_progress(connection, auth.0, period_start, period_end);
    let reading_days = calculate_reading_days(connection, auth.0, period_start, period_end);
    let streak_reference = std::cmp::min(today, period_end);
    let reading_streak_days = calculate_reading_streak(connection, auth.0, streak_reference);
    let average_rating = calculate_average_rating(connection, auth.0, period_start, period_end);

    let books_added: i64 = match books
        .filter(schema::books::dsl::user.eq(auth.0))
        .filter(
            schema::books::dsl::added_at
                .ge(period_start.and_hms_opt(0, 0, 0).unwrap().and_utc()),
        )
        .filter(
            schema::books::dsl::added_at
                .le(period_end.and_hms_opt(23, 59, 59).unwrap().and_utc()),
        )
        .count()
        .get_result(connection)
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error counting added books: {}", e)
                })),
            )
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "mode": payload.mode,
            "year": payload.year,
            "month": payload.month,
            "books_read": books_read,
            "pages_read": pages_read,
            "books_added": books_added,
            "reading_days": reading_days,
            "reading_streak_days": reading_streak_days,
            "average_rating": average_rating.map(|r| (r * 10.0).round() / 10.0),
        })),
    )
}
