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

type ApiResponse = (StatusCode, Json<serde_json::Value>);

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/reading", post(get_reading_info))
        .route("/api/books/start-reading", post(start_reading_session))
        .route("/api/books/track-progress", post(track_progress))
        .route("/api/readings/active", post(list_active_readings))
        .route("/api/readings/delete", post(delete_reading))
        .route("/api/readings/update-started-at", post(update_started_at))
}

#[derive(Debug, Deserialize)]
pub struct ReadingRequest {
    pub reading_id: String,
}

#[derive(Debug, Deserialize)]
pub struct StartReadingRequest {
    pub book_id: String,
    pub total_pages: i32,
    pub started_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrackProgressRequest {
    pub reading_id: String,
    pub progress: i32,
    pub read_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStartedAtRequest {
    pub reading_id: String,
    pub started_at: String,
}

fn parse_uuid(id_str: &str, kind: &str) -> Result<Uuid, ApiResponse> {
    Uuid::parse_str(id_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse {
                error: format!("Invalid {}.", kind)
            })),
        )
    })
}

fn find_reading_for_user(
    reading_id: Uuid,
    auth: &AuthUser,
    connection: &mut PgConnection,
) -> Result<Reading, ApiResponse> {
    let reading = readings
        .filter(schema::readings::dsl::id.eq(reading_id))
        .first::<Reading>(connection)
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(json!(ErrorResponse {
                    error: "Reading not found.".to_string()
                })),
            )
        })?;

    if reading.user != auth.0 {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ErrorResponse {
                error: "Access denied.".to_string()
            })),
        ));
    }

    Ok(reading)
}

pub(crate) async fn get_reading_info(
    auth: AuthUser,
    Json(payload): Json<ReadingRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let reading_id = match parse_uuid(&payload.reading_id, "reading ID") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let reading = match find_reading_for_user(reading_id, &auth, connection) {
        Ok(r) => r,
        Err(e) => return e,
    };

    let db_entries = match reading_entries
        .filter(crate::schema::reading_entries::dsl::reading.eq(reading_id))
        .load::<ReadingEntry>(connection)
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error loading entries: {}", e)
                })),
            )
        }
    };

    let json_entries: Vec<_> = db_entries
        .iter()
        .map(|entry| {
            json!({
                "id": entry.id.to_string(),
                "progress": entry.progress,
                "mode": entry.mode.to_string(),
                "read_at": entry.read_at.to_string(),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "book_id": reading.book.to_string(),
            "started_at": reading.started_at.to_string(),
            "total_pages": reading.total_pages,
            "entries": json_entries,
        })),
    )
}

pub(crate) async fn start_reading_session(
    auth: AuthUser,
    Json(payload): Json<StartReadingRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();
    let book_id = match parse_uuid(&payload.book_id, "book ID") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let book: crate::models::Book = match crate::schema::books::dsl::books
        .filter(crate::schema::books::dsl::id.eq(book_id))
        .first(connection)
    {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!(ErrorResponse {
                    error: "Book not found.".to_string()
                })),
            )
        }
    };

    if book.user != auth.0 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!(ErrorResponse {
                error: "Access denied.".to_string()
            })),
        );
    }

    let started_at = match payload.started_at {
        Some(ref s) => match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ErrorResponse {
                        error: "Invalid date format. Use YYYY-MM-DD.".to_string()
                    })),
                )
            }
        },
        None => chrono::Utc::now().date_naive(),
    };

    let new_reading = Reading {
        id: Uuid::new_v4(),
        book: book_id,
        user: auth.0,
        total_pages: payload.total_pages,
        progress: 0,
        mode: ReadingMode::Pages,
        started_at,
        finished_at: None,
        cancelled_at: None,
        updated_at: chrono::Utc::now().naive_utc(),
        created_at: chrono::Utc::now().naive_utc(),
    };

    match diesel::insert_into(schema::readings::dsl::readings)
        .values(&new_reading)
        .execute(connection)
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Reading session started successfully." })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error while starting the reading session: {}", e)
            })),
        ),
    }
}

pub(crate) async fn track_progress(
    auth: AuthUser,
    Json(payload): Json<TrackProgressRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let reading_id = match parse_uuid(&payload.reading_id, "reading ID") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let reading = match find_reading_for_user(reading_id, &auth, connection) {
        Ok(r) => r,
        Err(e) => return e,
    };

    let new_entry = ReadingEntry {
        id: Uuid::new_v4(),
        reading: reading_id,
        book: reading.book,
        user: auth.0,
        progress: payload.progress,
        mode: ReadingMode::Pages,
        read_at: match chrono::NaiveDate::parse_from_str(&payload.read_at, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ErrorResponse {
                        error: "Invalid date format. Use YYYY-MM-DD.".to_string()
                    })),
                )
            }
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
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Progress tracked successfully." })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error while tracking progress: {}", e)
            })),
        ),
    }
}

pub(crate) async fn list_active_readings(auth: AuthUser) -> impl IntoResponse {
    let connection = &mut connect();

    type Row = (
        uuid::Uuid,
        i32,
        i32,
        uuid::Uuid,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );

    let rows = readings
        .inner_join(books)
        .filter(schema::readings::dsl::user.eq(auth.0))
        .filter(schema::readings::dsl::finished_at.is_null())
        .filter(schema::readings::dsl::cancelled_at.is_null())
        .order(schema::readings::dsl::updated_at.desc())
        .select((
            schema::readings::dsl::id,
            schema::readings::dsl::progress,
            schema::readings::dsl::total_pages,
            schema::books::dsl::id,
            schema::books::dsl::title,
            schema::books::dsl::author,
            schema::books::dsl::google_books_id,
            schema::books::dsl::isbn13,
            schema::books::dsl::isbn10,
            schema::books::dsl::cover_url,
        ))
        .load::<Row>(connection);

    match rows {
        Ok(rows) => {
            let items: Vec<_> = rows
                .into_iter()
                .map(
                    |(
                        reading_id,
                        progress,
                        total_pages,
                        book_id,
                        title,
                        author,
                        google_books_id,
                        isbn13,
                        isbn10,
                        cover_url,
                    )| {
                        json!({
                            "reading_id": reading_id.to_string(),
                            "book_id": book_id.to_string(),
                            "title": title,
                            "author": author,
                            "google_books_id": google_books_id,
                            "isbn13": isbn13,
                            "isbn10": isbn10,
                            "progress": progress,
                            "total_pages": total_pages,
                            "cover_url": cover_url,
                        })
                    },
                )
                .collect();
            (StatusCode::OK, Json(json!({ "readings": items })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error loading active readings: {}", e)
            })),
        ),
    }
}

pub(crate) async fn update_started_at(
    auth: AuthUser,
    Json(payload): Json<UpdateStartedAtRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let reading_id = match parse_uuid(&payload.reading_id, "reading ID") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let started_at = match chrono::NaiveDate::parse_from_str(&payload.started_at, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid date format. Use YYYY-MM-DD.".to_string()
                })),
            )
        }
    };

    if let Err(e) = find_reading_for_user(reading_id, &auth, connection) {
        return e;
    }

    match diesel::update(readings.filter(schema::readings::dsl::id.eq(reading_id)))
        .set((
            schema::readings::dsl::started_at.eq(started_at),
            schema::readings::dsl::updated_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .execute(connection)
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Start date updated successfully." })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error updating start date: {}", e)
            })),
        ),
    }
}

pub(crate) async fn delete_reading(
    auth: AuthUser,
    Json(payload): Json<ReadingRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let reading_id = match parse_uuid(&payload.reading_id, "reading ID") {
        Ok(id) => id,
        Err(e) => return e,
    };

    if let Err(e) = find_reading_for_user(reading_id, &auth, connection) {
        return e;
    }

    let result = connection.transaction::<_, diesel::result::Error, _>(|connection| {
        diesel::delete(
            reading_entries.filter(crate::schema::reading_entries::dsl::reading.eq(reading_id)),
        )
        .execute(connection)?;
        diesel::delete(readings.filter(schema::readings::dsl::id.eq(reading_id)))
            .execute(connection)?;
        Ok(())
    });

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Reading deleted successfully." })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error deleting reading: {}", e)
            })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::post, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_start_reading_requires_auth() {
        let app = Router::new().route("/api/books/start-reading", post(start_reading_session));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/start-reading")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_track_progress_requires_auth() {
        let app = Router::new().route("/api/books/track-progress", post(track_progress));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/track-progress")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_reading_info_requires_auth() {
        let app = Router::new().route("/api/books/reading", post(get_reading_info));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/reading")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_delete_reading_requires_auth() {
        let app = Router::new().route("/api/readings/delete", post(delete_reading));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/readings/delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
