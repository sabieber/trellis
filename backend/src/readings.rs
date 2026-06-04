use crate::auth::AuthUser;
use crate::db::connect;
use crate::models::{Reading, ReadingEntry, ReadingMode};
use crate::schema::books::dsl::books;
use crate::schema::reading_entries::dsl::reading_entries;
use crate::schema::readings::dsl::readings;
use crate::{schema, ErrorResponse};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/reading", post(get_reading_info))
        .route("/api/books/start-reading", post(start_reading_session))
        .route("/api/books/track-progress", post(track_progress))
        .route("/api/readings/active", post(list_active_readings))
}

/// Request type for getting information about a reading session.
#[derive(Debug, Deserialize)]
pub struct ReadingInfoRequest {
    pub reading_id: String,
}

/// Fetches reading session information by reading ID.
///
/// This route accepts a JSON payload with the following structure:
/// - `reading_id`: The UUID of the reading session to fetch information for.
pub(crate) async fn get_reading_info(
    auth: AuthUser,
    Json(payload): Json<ReadingInfoRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let reading_id = match Uuid::parse_str(&payload.reading_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid reading ID.".to_string() }))),
    };

    let reading = match readings
        .filter(schema::readings::dsl::id.eq(reading_id))
        .first::<Reading>(connection)
    {
        Ok(r) => r,
        Err(_) => return (StatusCode::NOT_FOUND, Json(json!(ErrorResponse { error: "Reading not found.".to_string() }))),
    };

    if reading.user != auth.0 {
        return (StatusCode::FORBIDDEN, Json(json!(ErrorResponse { error: "Access denied.".to_string() })));
    }

    let db_entries = match reading_entries
        .filter(crate::schema::reading_entries::dsl::reading.eq(reading_id))
        .load::<ReadingEntry>(connection)
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Error loading entries: {}", e) }))),
    };

    let mut json_entries = Vec::new();
    for entry in db_entries {
        let json_entry = json!({
            "id": entry.id.to_string(),
            "progress": entry.progress,
            "mode": entry.mode.to_string(),
            "read_at": entry.read_at.to_string(),
        });
        json_entries.push(json_entry);
    }

    (
        StatusCode::OK,
        Json(json!({
            "book_id": reading.book.to_string(),
            "entries": json_entries,
        })),
    )
}

/// Request type for starting a new reading session.
#[derive(Debug, Deserialize)]
pub struct StartReadingRequest {
    pub book_id: String,
    pub total_pages: i32,
}

/// Starts a new reading session for a book.
///
/// This route accepts a JSON payload with the following structure:
/// - `book_id`: The UUID of the book to start reading.
/// - `total_pages`: The total number of pages of the book.
pub(crate) async fn start_reading_session(
    auth: AuthUser,
    Json(payload): Json<StartReadingRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();
    let book_id = match Uuid::parse_str(&payload.book_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid book ID.".to_string() }))),
    };

    let book: crate::models::Book = match crate::schema::books::dsl::books
        .filter(crate::schema::books::dsl::id.eq(book_id))
        .first(connection)
    {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, Json(json!(ErrorResponse { error: "Book not found.".to_string() }))),
    };

    if book.user != auth.0 {
        return (StatusCode::FORBIDDEN, Json(json!(ErrorResponse { error: "Access denied.".to_string() })));
    }

    let new_reading = Reading {
        id: Uuid::new_v4(),
        book: book_id,
        user: auth.0,
        total_pages: payload.total_pages,
        progress: 0,
        mode: ReadingMode::Pages,
        started_at: chrono::Utc::now().date_naive(),
        finished_at: None,
        cancelled_at: None,
        updated_at: chrono::Utc::now().naive_utc(),
        created_at: chrono::Utc::now().naive_utc(),
    };

    match diesel::insert_into(schema::readings::dsl::readings)
        .values(&new_reading)
        .execute(connection)
    {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "message": "Reading session started successfully." }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Error while starting the reading session: {}", e) }))),
    }
}

/// Request type for tracking progress in a reading session.
#[derive(Debug, Deserialize)]
pub struct TrackProgressRequest {
    pub reading_id: String,
    pub progress: i32,
    pub read_at: String,
}

/// Tracks progress for a reading session.
///
/// This route accepts a JSON payload with the following structure:
/// - `reading_id`: The UUID of the reading session.
/// - `progress`: The page number reached.
/// - `read_at`: The date when reading took place.
pub(crate) async fn track_progress(
    auth: AuthUser,
    Json(payload): Json<TrackProgressRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let reading_id = match Uuid::parse_str(&payload.reading_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid reading ID.".to_string() }))),
    };

    // Verify the reading belongs to the authenticated user
    let reading: Reading = match readings
        .filter(schema::readings::dsl::id.eq(reading_id))
        .first(connection)
    {
        Ok(r) => r,
        Err(_) => return (StatusCode::NOT_FOUND, Json(json!(ErrorResponse { error: "Reading not found.".to_string() }))),
    };

    if reading.user != auth.0 {
        return (StatusCode::FORBIDDEN, Json(json!(ErrorResponse { error: "Access denied.".to_string() })));
    }

    let new_entry = ReadingEntry {
        id: Uuid::new_v4(),
        reading: reading_id,
        book: reading.book,
        user: auth.0,
        progress: payload.progress,
        mode: ReadingMode::Pages,
        read_at: match chrono::NaiveDate::parse_from_str(&payload.read_at, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid date format. Use YYYY-MM-DD.".to_string() }))),
        },
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    let transaction_result = connection.transaction::<_, diesel::result::Error, _>(|connection| {
        diesel::insert_into(reading_entries)
            .values(&new_entry)
            .execute(connection)?;

        diesel::update(readings.filter(schema::readings::dsl::id.eq(reading_id)))
            .set(schema::readings::dsl::progress.eq(payload.progress))
            .execute(connection)?;

        if payload.progress >= reading.total_pages {
            diesel::update(readings.filter(schema::readings::dsl::id.eq(reading_id)))
                .set(schema::readings::dsl::finished_at.eq(new_entry.read_at))
                .execute(connection)?;
        }

        Ok(())
    });

    match transaction_result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "message": "Progress tracked successfully." }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Error while tracking progress: {}", e) }))),
    }
}

pub(crate) async fn list_active_readings(auth: AuthUser) -> impl IntoResponse {
    let connection = &mut connect();

    type Row = (uuid::Uuid, i32, i32, uuid::Uuid, Option<String>, Option<String>, Option<String>);

    let rows = readings
        .inner_join(books)
        .filter(schema::readings::dsl::user.eq(auth.0))
        .filter(schema::readings::dsl::finished_at.is_null())
        .filter(schema::readings::dsl::cancelled_at.is_null())
        .select((
            schema::readings::dsl::id,
            schema::readings::dsl::progress,
            schema::readings::dsl::total_pages,
            schema::books::dsl::id,
            schema::books::dsl::title,
            schema::books::dsl::author,
            schema::books::dsl::google_books_id,
        ))
        .load::<Row>(connection);

    match rows {
        Ok(rows) => {
            let items: Vec<_> = rows.into_iter().map(|(reading_id, progress, total_pages, book_id, title, author, google_books_id)| {
                json!({
                    "reading_id": reading_id.to_string(),
                    "book_id": book_id.to_string(),
                    "title": title,
                    "author": author,
                    "google_books_id": google_books_id,
                    "progress": progress,
                    "total_pages": total_pages,
                })
            }).collect();
            (StatusCode::OK, Json(json!({ "readings": items })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Error loading active readings: {}", e) }))),
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use tower::ServiceExt;
    use super::*;

    #[tokio::test]
    async fn test_start_reading_requires_auth() {
        let app = Router::new().route("/api/books/start-reading", post(start_reading_session));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/start-reading").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_track_progress_requires_auth() {
        let app = Router::new().route("/api/books/track-progress", post(track_progress));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/track-progress").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_reading_info_requires_auth() {
        let app = Router::new().route("/api/books/reading", post(get_reading_info));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/reading").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
