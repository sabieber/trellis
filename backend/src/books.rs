use crate::auth::AuthUser;
use crate::db::connect;
use crate::models::{Book, Reading};
use crate::schema::books::dsl::books;
use crate::schema::readings::dsl::readings;
use crate::{schema, ErrorResponse};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/info", post(get_book_info))
        .route("/api/books/resolve-google-id", post(resolve_google_id))
}

/// Request type for getting information about a book.
#[derive(Debug, Deserialize)]
pub struct BookInfoRequest {
    pub book_id: String,
}

/// Response type for book information.
#[derive(Debug, Serialize)]
pub struct BookInfoResponse {
    pub google_books_id: Option<String>,
    pub isbn13: Option<String>,
    pub readings: Vec<serde_json::Value>,
}

/// Fetches book information by book ID.
///
/// This route accepts a JSON payload with the following structure:
/// - `book_id`: The UUID of the book to fetch information for.
pub(crate) async fn get_book_info(
    auth: AuthUser,
    Json(payload): Json<BookInfoRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let book_id = match Uuid::parse_str(&payload.book_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid book ID.".to_string() }))),
    };

    let db_readings = match readings
        .filter(schema::readings::dsl::book.eq(book_id))
        .filter(schema::readings::dsl::user.eq(auth.0))
        .load::<Reading>(connection)
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Error loading readings: {}", e) }))),
    };

    let mut json_readings = Vec::new();
    for reading in db_readings {
        let json_reading = json!({
            "id": reading.id.to_string(),
            "total_pages": reading.total_pages,
            "progress": reading.progress,
            "mode": reading.mode.to_string(),
            "started_at": reading.started_at.to_string(),
            "finished_at": reading.finished_at.map(|d| d.to_string()),
            "cancelled_at": reading.cancelled_at.map(|d| d.to_string()),
        });
        json_readings.push(json_reading);
    }

    match books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(auth.0))
        .first::<Book>(connection)
    {
        Ok(book) => (
            StatusCode::OK,
            Json(json!(BookInfoResponse {
                google_books_id: book.google_books_id,
                isbn13: book.isbn13,
                readings: json_readings,
            })),
        ),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "Book not found.".to_string(),
            })),
        ),
    }
}

/// Request type for resolving a Google Books ID.
#[derive(Debug, Deserialize)]
pub struct ResolveGoogleIdRequest {
    pub book_id: String,
}

/// Resolves the Google Books ID for a book.
///
/// This route accepts a JSON payload with the following structure:
/// - `book_id`: The UUID of the book to resolve the Google Books ID for.
///
/// If the book already has a `google_books_id` stored, it is returned immediately.
/// Otherwise, the ISBN is used to look up the ID via the Google Books API.
/// On success, the resolved ID is persisted to the database.
pub(crate) async fn resolve_google_id(
    auth: AuthUser,
    Json(payload): Json<ResolveGoogleIdRequest>,
) -> impl IntoResponse {
    let book_id = match Uuid::parse_str(&payload.book_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "google_books_id": serde_json::Value::Null })),
            )
        }
    };

    let connection = &mut connect();

    let book = match books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(auth.0))
        .first::<Book>(connection)
    {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "google_books_id": serde_json::Value::Null })),
            )
        }
    };

    if let Some(gid) = book.google_books_id {
        return (StatusCode::OK, Json(json!({ "google_books_id": gid })));
    }

    let isbn = book
        .isbn13
        .filter(|s| !s.is_empty())
        .or(book.isbn10.filter(|s| !s.is_empty()));

    let Some(isbn) = isbn else {
        return (
            StatusCode::OK,
            Json(json!({ "google_books_id": serde_json::Value::Null })),
        );
    };

    let client = reqwest::Client::new();
    let google_id = crate::google_books_client::lookup_id_by_isbn(&client, &isbn).await;

    if let Some(ref gid) = google_id {
        let _ = diesel::update(
            books
                .filter(schema::books::dsl::id.eq(book_id))
                .filter(schema::books::dsl::user.eq(auth.0)),
        )
        .set(schema::books::dsl::google_books_id.eq(gid))
        .execute(connection);
    }

    (
        StatusCode::OK,
        Json(json!({ "google_books_id": google_id })),
    )
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use tower::ServiceExt;
    use super::*;

    #[tokio::test]
    async fn test_get_book_info_requires_auth() {
        let app = Router::new().route("/api/books/info", post(get_book_info));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/info").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_resolve_google_id_requires_auth() {
        let app = Router::new().route("/api/books/resolve-google-id", post(resolve_google_id));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/resolve-google-id").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
