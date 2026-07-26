use crate::auth::AuthUser;
use crate::db::connect;
use crate::goals::{calculate_books_progress, calculate_pages_progress, estimate_pages};
use crate::models::ReadingMode;
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
    router
        .route("/api/stats/overview", post(overview))
        .route("/api/stats/activity", post(activity))
        .route("/api/stats/breakdown", post(breakdown))
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

/// Resolves the reporting period of a request. Returns an error message when
/// the requested period is invalid or lies in the future. The `"total"` mode is
/// resolved separately by [`resolve_request_period`], since it needs the user's
/// data to find where the span starts.
fn resolve_period(
    mode: &str,
    year: i32,
    month: Option<u32>,
    today: NaiveDate,
) -> Result<(NaiveDate, NaiveDate), String> {
    if year < 1970 || year > today.year() {
        return Err("Invalid year.".to_string());
    }

    match mode {
        "year" => Ok(year_period(year)),
        "month" => match month {
            Some(month) if (1..=12).contains(&month) => {
                let period = month_period(year, month);
                if period.0 > today {
                    Err("Month must not be in the future.".to_string())
                } else {
                    Ok(period)
                }
            }
            _ => Err("Invalid month. Must be between 1 and 12.".to_string()),
        },
        _ => Err("Invalid mode. Must be 'year', 'month', or 'total'.".to_string()),
    }
}

/// The user's earliest data point across every field the stats aggregate on:
/// a reading's start (bounds finished/abandoned/in-progress readings too, as it
/// is never null), a logged reading day, and a book's add date. Anchors the
/// `"total"` period so its span stays tight — reaching back only as far as the
/// user's oldest data, not to some arbitrary epoch — while still covering
/// everything. Returns `None` only for an entirely empty account.
fn earliest_activity_date(connection: &mut PgConnection, user_id: Uuid) -> Option<NaiveDate> {
    let started: Option<NaiveDate> = readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .select(diesel::dsl::min(schema::readings::dsl::started_at))
        .first(connection)
        .ok()
        .flatten();
    let read: Option<NaiveDate> = reading_entries
        .filter(schema::reading_entries::dsl::user.eq(user_id))
        .select(diesel::dsl::min(schema::reading_entries::dsl::read_at))
        .first(connection)
        .ok()
        .flatten();
    let added: Option<NaiveDate> = books
        .filter(schema::books::dsl::user.eq(user_id))
        .select(diesel::dsl::min(schema::books::dsl::added_at))
        .first::<Option<chrono::DateTime<chrono::Utc>>>(connection)
        .ok()
        .flatten()
        .map(|dt| dt.date_naive());
    [started, read, added].into_iter().flatten().min()
}

/// Resolves the period for an incoming request. Delegates to [`resolve_period`]
/// for `"year"`/`"month"` and handles `"total"` here, where the span runs from
/// the user's first reading up to today.
fn resolve_request_period(
    connection: &mut PgConnection,
    user_id: Uuid,
    mode: &str,
    year: i32,
    month: Option<u32>,
    today: NaiveDate,
) -> Result<(NaiveDate, NaiveDate), String> {
    if mode == "total" {
        return Ok(total_period(earliest_activity_date(connection, user_id), today));
    }
    resolve_period(mode, year, month, today)
}

/// The `"total"` span: from the user's earliest data point up to today, or a
/// zero-width today..today span for an account with no data yet.
fn total_period(earliest: Option<NaiveDate>, today: NaiveDate) -> (NaiveDate, NaiveDate) {
    (earliest.unwrap_or(today), today)
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

/// Averages the ratings of distinct books finished within the period, derived
/// from the rating distribution (its weighted mean). Returns `None` when no
/// finished book has a rating.
fn calculate_average_rating(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Option<f64> {
    let distribution =
        calculate_rating_distribution(connection, user_id, period_start, period_end);

    let count: i64 = distribution.iter().sum();
    if count == 0 {
        return None;
    }

    let weighted: i64 = distribution
        .iter()
        .enumerate()
        .map(|(index, &n)| (index as i64 + 1) * n)
        .sum();
    Some(weighted as f64 / count as f64)
}

/// Averages the calendar days between start and finish of the readings finished
/// within the period. Returns `None` when nothing was finished.
fn calculate_avg_days_to_finish(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Option<f64> {
    let rows: Vec<(NaiveDate, Option<NaiveDate>)> = readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.is_not_null())
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .select((
            schema::readings::dsl::started_at,
            schema::readings::dsl::finished_at,
        ))
        .load(connection)
        .ok()?;

    let spans: Vec<i64> = rows
        .into_iter()
        .filter_map(|(started, finished)| finished.map(|f| (f - started).num_days().max(0)))
        .collect();

    if spans.is_empty() {
        return None;
    }
    Some(spans.iter().sum::<i64>() as f64 / spans.len() as f64)
}

/// Distribution of the 1–5 star ratings of the distinct books finished within
/// the period, indexed 0 (one star) to 4 (five stars). Re-reads count once and
/// unrated books are ignored, mirroring [`calculate_average_rating`].
fn calculate_rating_distribution(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> [i64; 5] {
    let rows: Vec<(Uuid, Option<i16>)> = readings
        .inner_join(books)
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.is_not_null())
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .select((schema::readings::dsl::book, schema::books::dsl::rating))
        .load(connection)
        .unwrap_or_default();

    let ratings: HashMap<Uuid, i16> = rows
        .into_iter()
        .filter_map(|(book, rating)| rating.map(|r| (book, r)))
        .collect();

    let mut distribution = [0i64; 5];
    for rating in ratings.values() {
        if (1..=5).contains(rating) {
            distribution[(rating - 1) as usize] += 1;
        }
    }
    distribution
}

/// The authors of the readings finished within the period, ranked by finished
/// books (re-reads count separately, like `books_read`) and capped at five.
/// Books without an author collapse into an "Unknown author" bucket.
fn calculate_top_authors(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Vec<(String, i64, i64)> {
    let rows: Vec<(Option<String>, i32)> = readings
        .inner_join(books)
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.is_not_null())
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .select((
            schema::books::dsl::author,
            schema::readings::dsl::total_pages,
        ))
        .load(connection)
        .unwrap_or_default();

    let mut per_author: HashMap<String, (i64, i64)> = HashMap::new();
    for (author, pages) in rows {
        let name = author
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| "Unknown author".to_string());
        let entry = per_author.entry(name).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += pages.max(0) as i64;
    }

    let mut authors: Vec<(String, i64, i64)> = per_author
        .into_iter()
        .map(|(name, (count, pages))| (name, count, pages))
        .collect();
    // Most books first, then most pages, then alphabetically for a stable order.
    authors.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(a.0.cmp(&b.0)));
    authors.truncate(5);
    authors
}

/// Counts the readings by outcome within the period: finished (finished in the
/// period), abandoned (cancelled in the period) and reading (started in the
/// period and still open). The three buckets are disjoint.
fn calculate_reading_states(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> (i64, i64, i64) {
    let finished: i64 = readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .count()
        .get_result(connection)
        .unwrap_or(0);

    let abandoned: i64 = readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::cancelled_at.ge(period_start))
        .filter(schema::readings::dsl::cancelled_at.le(period_end))
        .count()
        .get_result(connection)
        .unwrap_or(0);

    let reading: i64 = readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::started_at.ge(period_start))
        .filter(schema::readings::dsl::started_at.le(period_end))
        .filter(schema::readings::dsl::finished_at.is_null())
        .filter(schema::readings::dsl::cancelled_at.is_null())
        .count()
        .get_result(connection)
        .unwrap_or(0);

    (finished, reading, abandoned)
}

/// Aggregates the pages logged per day within the period.
///
/// Reading entries carry the *cumulative* progress of their reading, so the
/// pages of a day are the positive delta towards the previous entry of the same
/// reading. Entries before the period are only read to establish that baseline.
/// This mirrors [`calculate_pages_progress`], hence the daily values add up to
/// the `pages_read` of the overview.
fn calculate_daily_pages(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> HashMap<NaiveDate, i64> {
    let mut per_day: HashMap<NaiveDate, i64> = HashMap::new();

    // join the book for its page count, so percentage entries convert to estimated
    // pages the same way [`calculate_pages_progress`] does
    let entries: Vec<(Uuid, NaiveDate, i32, ReadingMode, Option<i32>)> = match reading_entries
        .inner_join(books)
        .filter(schema::reading_entries::dsl::user.eq(user_id))
        .filter(schema::reading_entries::dsl::read_at.le(period_end))
        .order((
            schema::reading_entries::dsl::reading.asc(),
            schema::reading_entries::dsl::read_at.asc(),
            schema::reading_entries::dsl::created_at.asc(),
        ))
        .select((
            schema::reading_entries::dsl::reading,
            schema::reading_entries::dsl::read_at,
            schema::reading_entries::dsl::progress,
            schema::reading_entries::dsl::mode,
            schema::books::dsl::page_count,
        ))
        .load(connection)
    {
        Ok(e) => e,
        Err(_) => return per_day,
    };

    let mut current_reading: Option<Uuid> = None;
    let mut previous_progress: i64 = 0;

    for (reading_id, read_at, progress, mode, page_count) in entries {
        if current_reading != Some(reading_id) {
            current_reading = Some(reading_id);
            previous_progress = 0;
        }

        let progress = estimate_pages(progress, &mode, page_count);
        if read_at >= period_start && progress > previous_progress {
            *per_day.entry(read_at).or_insert(0) += progress - previous_progress;
        }
        previous_progress = progress;
    }

    per_day
}

/// Counts the readings finished per day within the period. Re-reads of the same
/// book count separately, just like [`calculate_books_progress`].
fn calculate_daily_books(
    connection: &mut PgConnection,
    user_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> HashMap<NaiveDate, i64> {
    let mut per_day: HashMap<NaiveDate, i64> = HashMap::new();

    let finished_dates: Vec<Option<NaiveDate>> = match readings
        .filter(schema::readings::dsl::user.eq(user_id))
        .filter(schema::readings::dsl::finished_at.is_not_null())
        .filter(schema::readings::dsl::finished_at.ge(period_start))
        .filter(schema::readings::dsl::finished_at.le(period_end))
        .select(schema::readings::dsl::finished_at)
        .load(connection)
    {
        Ok(d) => d,
        Err(_) => return per_day,
    };

    for finished_at in finished_dates.into_iter().flatten() {
        *per_day.entry(finished_at).or_insert(0) += 1;
    }

    per_day
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

    let bad_request = |message: String| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse { error: message })),
        )
    };

    let (period_start, period_end) = match resolve_request_period(
        connection,
        auth.0,
        &payload.mode,
        payload.year,
        payload.month,
        today,
    ) {
        Ok(period) => period,
        Err(message) => return bad_request(message),
    };

    let books_read = calculate_books_progress(connection, auth.0, period_start, period_end);
    let pages_read = calculate_pages_progress(connection, auth.0, period_start, period_end);
    let reading_days = calculate_reading_days(connection, auth.0, period_start, period_end);
    let streak_reference = std::cmp::min(today, period_end);
    let reading_streak_days = calculate_reading_streak(connection, auth.0, streak_reference);
    let average_rating = calculate_average_rating(connection, auth.0, period_start, period_end);
    let avg_days_to_finish =
        calculate_avg_days_to_finish(connection, auth.0, period_start, period_end);

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
            "period_start": period_start.to_string(),
            "period_end": period_end.to_string(),
            "books_read": books_read,
            "pages_read": pages_read,
            "books_added": books_added,
            "reading_days": reading_days,
            "reading_streak_days": reading_streak_days,
            "average_rating": average_rating.map(|r| (r * 10.0).round() / 10.0),
            "avg_days_to_finish": avg_days_to_finish.map(|d| (d * 10.0).round() / 10.0),
        })),
    )
}

/// Request type for the reading activity time series.
#[derive(Debug, Deserialize)]
pub struct ActivityRequest {
    pub mode: String,
    pub year: i32,
    pub month: Option<u32>,
}

/// Returns the day-by-day reading activity of a year or month for the
/// authenticated user.
///
/// This route accepts a JSON payload with the following structure:
/// - `mode`: Either `"year"` or `"month"`.
/// - `year`: The year to report on (must not be in the future).
/// - `month`: The month (1-12) to report on, required for mode `"month"`.
///
/// The response contains the requested period, its `start` and `end` date and
/// the `days` of the period that saw activity, ascending by date. Days without
/// any activity are omitted, so consumers have to fill the gaps themselves:
/// - `date`: The day in `YYYY-MM-DD` format.
/// - `pages`: Pages logged on that day.
/// - `books`: Readings finished on that day.
pub(crate) async fn activity(
    auth: AuthUser,
    Json(payload): Json<ActivityRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let today = chrono::Utc::now().date_naive();

    let (period_start, period_end) = match resolve_request_period(
        connection,
        auth.0,
        &payload.mode,
        payload.year,
        payload.month,
        today,
    ) {
        Ok(period) => period,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse { error: message })),
            )
        }
    };

    let pages_per_day = calculate_daily_pages(connection, auth.0, period_start, period_end);
    let books_per_day = calculate_daily_books(connection, auth.0, period_start, period_end);

    let mut active_days: Vec<NaiveDate> = pages_per_day
        .keys()
        .chain(books_per_day.keys())
        .copied()
        .collect::<HashSet<NaiveDate>>()
        .into_iter()
        .collect();
    active_days.sort_unstable();

    let days: Vec<serde_json::Value> = active_days
        .into_iter()
        .map(|date| {
            json!({
                "date": date.to_string(),
                "pages": pages_per_day.get(&date).copied().unwrap_or(0),
                "books": books_per_day.get(&date).copied().unwrap_or(0),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "mode": payload.mode,
            "year": payload.year,
            "month": payload.month,
            "start": period_start.to_string(),
            "end": period_end.to_string(),
            "days": days,
        })),
    )
}

/// Request type for the stats breakdown.
#[derive(Debug, Deserialize)]
pub struct BreakdownRequest {
    pub mode: String,
    pub year: i32,
    pub month: Option<u32>,
}

/// Returns aggregate breakdowns of a year or month for the authenticated user.
///
/// This route accepts the same JSON payload as the overview and responds with:
/// - `rating_distribution`: counts of finished books per star rating, index 0
///   (one star) to 4 (five stars)
/// - `top_authors`: up to five authors of finished books, each with the number of
///   finished `books` and their summed `pages`, most read first
/// - `reading_states`: readings by outcome in the period (`finished`, `reading`,
///   `abandoned`)
pub(crate) async fn breakdown(
    auth: AuthUser,
    Json(payload): Json<BreakdownRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let today = chrono::Utc::now().date_naive();

    let (period_start, period_end) = match resolve_request_period(
        connection,
        auth.0,
        &payload.mode,
        payload.year,
        payload.month,
        today,
    ) {
        Ok(period) => period,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse { error: message })),
            )
        }
    };

    let rating_distribution =
        calculate_rating_distribution(connection, auth.0, period_start, period_end);
    let top_authors = calculate_top_authors(connection, auth.0, period_start, period_end);
    let (finished, reading, abandoned) =
        calculate_reading_states(connection, auth.0, period_start, period_end);

    let authors: Vec<serde_json::Value> = top_authors
        .into_iter()
        .map(|(author, count, page_sum)| json!({"author": author, "books": count, "pages": page_sum}))
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "mode": payload.mode,
            "year": payload.year,
            "month": payload.month,
            "rating_distribution": rating_distribution,
            "top_authors": authors,
            "reading_states": {
                "finished": finished,
                "reading": reading,
                "abandoned": abandoned,
            },
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use chrono::NaiveDate;
    use tower::ServiceExt;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_resolve_period_year() {
        let today = date(2026, 7, 24);
        assert_eq!(
            resolve_period("year", 2025, None, today),
            Ok((date(2025, 1, 1), date(2025, 12, 31)))
        );
    }

    #[test]
    fn test_resolve_period_month_end_of_month() {
        let today = date(2026, 7, 24);
        assert_eq!(
            resolve_period("month", 2024, Some(2), today),
            Ok((date(2024, 2, 1), date(2024, 2, 29)))
        );
        assert_eq!(
            resolve_period("month", 2026, Some(7), today),
            Ok((date(2026, 7, 1), date(2026, 7, 31)))
        );
    }

    #[test]
    fn test_total_period() {
        let today = date(2026, 7, 26);
        // Spans from the earliest data point to today.
        assert_eq!(
            total_period(Some(date(2022, 12, 15)), today),
            (date(2022, 12, 15), today)
        );
        // Empty account collapses to a zero-width today..today span.
        assert_eq!(total_period(None, today), (today, today));
    }

    #[test]
    fn test_resolve_period_rejects_invalid_input() {
        let today = date(2026, 7, 24);
        assert!(resolve_period("year", 2027, None, today).is_err());
        assert!(resolve_period("year", 1969, None, today).is_err());
        assert!(resolve_period("month", 2026, None, today).is_err());
        assert!(resolve_period("month", 2026, Some(13), today).is_err());
        assert!(resolve_period("month", 2026, Some(8), today).is_err());
        assert!(resolve_period("week", 2026, None, today).is_err());
    }

    #[tokio::test]
    async fn test_overview_requires_auth() {
        let app = Router::new().route("/api/stats/overview", post(overview));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/stats/overview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_activity_requires_auth() {
        let app = Router::new().route("/api/stats/activity", post(activity));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/stats/activity")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_breakdown_requires_auth() {
        let app = Router::new().route("/api/stats/breakdown", post(breakdown));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/stats/breakdown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
