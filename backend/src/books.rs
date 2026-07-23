use crate::auth::AuthUser;
use crate::db::connect;
use crate::models::{Book, BookShelf, Reading};
use crate::schema::books::dsl::books;
use crate::schema::readings::dsl::readings;
use crate::{schema, ErrorResponse};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Trims a string and maps the empty result to `None`, so blank ISBNs/IDs are
/// stored as SQL NULL rather than `""` (which would collide under the partial
/// unique indexes).
fn normalize(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Resolves the canonical book row for a user using the identity ladder.
///
/// The ladder is: source id (`google_books_id`/`open_library_id`) → isbn13 →
/// isbn10 → title+author. A source id match is authoritative. When it misses,
/// we still fall back to ISBN/title matching, but a matched row is only reused
/// if it does NOT already carry a *different* source id of the same kind. That
/// distinguishes "the same book re-added" (the existing row has no or an equal
/// source id — merge onto it) from "two distinct books that legitimately share
/// an ISBN" (each carries its own source id, e.g. bundle editions — keep apart).
///
/// `page_count` is stored on insert and backfilled onto a matched row whose
/// `page_count` is still NULL; an existing value is never overwritten.
///
/// All lookups and the insert run on the passed connection, so within a single
/// transaction repeated calls for the same book converge on one row.
pub(crate) fn resolve_or_create_book(
    conn: &mut PgConnection,
    user_id: Uuid,
    title: Option<String>,
    author: Option<String>,
    isbn13: Option<String>,
    isbn10: Option<String>,
    google_books_id: Option<String>,
    open_library_id: Option<String>,
    added_at: chrono::NaiveDateTime,
    rating: Option<i16>,
    cover_url: Option<String>,
    page_count: Option<i32>,
) -> QueryResult<Uuid> {
    use crate::schema::books::dsl as b;

    let title = normalize(title);
    let author = normalize(author);
    let isbn13 = normalize(isbn13);
    let isbn10 = normalize(isbn10);
    let google_books_id = normalize(google_books_id);
    let open_library_id = normalize(open_library_id);
    let page_count = page_count.filter(|p| *p > 0);

    let base = || b::books.filter(b::user.eq(user_id)).into_boxed();

    // Returns the matched id, backfilling page_count when the row has none.
    let reuse = |conn: &mut PgConnection, id: Uuid| -> QueryResult<Uuid> {
        if let Some(p) = page_count {
            diesel::update(b::books.filter(b::id.eq(id)).filter(b::page_count.is_null()))
                .set(b::page_count.eq(p))
                .execute(conn)?;
        }
        Ok(id)
    };

    if let Some(ref gid) = google_books_id {
        if let Some(id) = base()
            .filter(b::google_books_id.eq(gid))
            .select(b::id)
            .first::<Uuid>(conn)
            .optional()?
        {
            return reuse(conn, id);
        }
    }
    if let Some(ref oid) = open_library_id {
        if let Some(id) = base()
            .filter(b::open_library_id.eq(oid))
            .select(b::id)
            .first::<Uuid>(conn)
            .optional()?
        {
            return reuse(conn, id);
        }
    }

    // Fallback matching when no source id matched above. Reuse a row found by
    // ISBN/title only if it does not already assert a *different* source id of
    // the same kind (a NULL or equal source id on the existing row means "same
    // book"; a differing one means a distinct edition that shares the ISBN).
    {
        let reusable = |rows: Vec<(Uuid, Option<String>, Option<String>)>| {
            rows.into_iter().find_map(|(id, ex_gid, ex_olid)| {
                let gid_conflict =
                    matches!((&ex_gid, &google_books_id), (Some(a), Some(b)) if a != b);
                let olid_conflict =
                    matches!((&ex_olid, &open_library_id), (Some(a), Some(b)) if a != b);
                (!gid_conflict && !olid_conflict).then_some(id)
            })
        };

        if let Some(ref v) = isbn13 {
            if let Some(id) = reusable(
                base()
                    .filter(b::isbn13.eq(v))
                    .select((b::id, b::google_books_id, b::open_library_id))
                    .load(conn)?,
            ) {
                return reuse(conn, id);
            }
        }
        if let Some(ref v) = isbn10 {
            if let Some(id) = reusable(
                base()
                    .filter(b::isbn10.eq(v))
                    .select((b::id, b::google_books_id, b::open_library_id))
                    .load(conn)?,
            ) {
                return reuse(conn, id);
            }
        }
        if let (Some(ref t), Some(ref a)) = (&title, &author) {
            if let Some(id) = reusable(
                base()
                    .filter(b::title.eq(t))
                    .filter(b::author.eq(a))
                    .select((b::id, b::google_books_id, b::open_library_id))
                    .load(conn)?,
            ) {
                return reuse(conn, id);
            }
        }
    }

    let new_id = Uuid::new_v4();
    let new_book = Book {
        id: new_id,
        user: user_id,
        title,
        author,
        isbn13,
        isbn10,
        google_books_id,
        open_library_id,
        added_at,
        rating,
        cover_url,
        page_count,
    };
    diesel::insert_into(b::books).values(&new_book).execute(conn)?;
    Ok(new_id)
}

/// Ensures a book is a member of a shelf. Idempotent: adding a book to a shelf it
/// already belongs to is a no-op.
pub(crate) fn ensure_membership(
    conn: &mut PgConnection,
    book_id: Uuid,
    shelf_id: Uuid,
    added_at: chrono::NaiveDateTime,
) -> QueryResult<()> {
    use crate::schema::book_shelves::dsl as bs;

    diesel::insert_into(bs::book_shelves)
        .values(&BookShelf {
            book: book_id,
            shelf: shelf_id,
            added_at,
        })
        .on_conflict((bs::book, bs::shelf))
        .do_nothing()
        .execute(conn)?;
    Ok(())
}

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/info", post(get_book_info))
        .route("/api/books/resolve-google-id", post(resolve_google_id))
        .route("/api/books/resolve-cover", post(resolve_cover))
        .route("/api/books/rate", post(rate_book))
        .route("/api/books/set-page-count", post(set_page_count))
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
    pub open_library_id: Option<String>,
    pub isbn13: Option<String>,
    pub isbn10: Option<String>,
    pub rating: Option<i16>,
    pub cover_url: Option<String>,
    pub page_count: Option<i32>,
    pub readings: Vec<serde_json::Value>,
    pub shelf_ids: Vec<String>,
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

    let shelf_ids: Vec<String> = schema::book_shelves::dsl::book_shelves
        .filter(schema::book_shelves::dsl::book.eq(book_id))
        .select(schema::book_shelves::dsl::shelf)
        .load::<Uuid>(connection)
        .unwrap_or_default()
        .into_iter()
        .map(|id| id.to_string())
        .collect();

    match books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(auth.0))
        .first::<Book>(connection)
    {
        Ok(book) => (
            StatusCode::OK,
            Json(json!(BookInfoResponse {
                google_books_id: book.google_books_id,
                open_library_id: book.open_library_id,
                isbn13: book.isbn13,
                isbn10: book.isbn10,
                rating: book.rating,
                cover_url: book.cover_url,
                page_count: book.page_count,
                readings: json_readings,
                shelf_ids,
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

/// Request type for resolving a book cover URL.
#[derive(Debug, Deserialize)]
pub struct ResolveCoverRequest {
    pub book_id: String,
}

/// Resolves and caches the cover URL for a book.
///
/// Accepts a JSON payload with `book_id` (the internal UUID).
/// Resolution strategy:
/// 1. If cached in DB, return immediately.
/// 2. Try Google Books API via ISBN.
/// 3. Try Open Library work detail via `open_library_id`.
/// 4. Try Open Library ISBN lookup → follow to work for cover.
/// 5. Fall back to the ISBN-based covers.openlibrary.org URL.
///
/// The resolved URL is persisted so subsequent calls are instant.
pub(crate) async fn resolve_cover(
    auth: AuthUser,
    Json(payload): Json<ResolveCoverRequest>,
) -> impl IntoResponse {
    let book_id = match Uuid::parse_str(&payload.book_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "cover_url": serde_json::Value::Null })),
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
                Json(json!({ "cover_url": serde_json::Value::Null })),
            )
        }
    };

    // 1. Return the cached value if present.
    if let Some(ref url) = book.cover_url {
        return (StatusCode::OK, Json(json!({ "cover_url": url })));
    }

    let client = reqwest::Client::new();
    let isbn = book
        .isbn13
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(book.isbn10.as_deref().filter(|s| !s.is_empty()));
    let mut resolved: Option<String> = None;

    // 2. Try Google Books API via ISBN.
    if resolved.is_none() {
        if let Some(isbn) = isbn {
            if let Some(gid) = crate::google_books_client::lookup_id_by_isbn(&client, isbn).await {
                resolved = Some(crate::google_books_client::cover_url_from_id(&gid));
            }
        }
    }

    // 3. Try Open Library work detail via stored open_library_id.
    if resolved.is_none() {
        if let Some(ref ol_id) = book.open_library_id {
            if let Some(nb) = crate::open_library_client::get_work(&client, ol_id).await {
                resolved = nb.cover_url;
            }
        }
    }

    // 4. Try Open Library ISBN lookup → follow to work.
    if resolved.is_none() {
        if let Some(isbn) = isbn {
            resolved = crate::open_library_client::resolve_cover_by_isbn(&client, isbn).await;
        }
    }

    // 5. Fall back to the ISBN-based covers URL as a last resort.
    if resolved.is_none() {
        resolved = isbn.map(crate::open_library_client::cover_url_from_isbn);
    }

    // Persist the resolved URL (even if None, to avoid re-probing).
    if let Some(ref url) = resolved {
        let _ = diesel::update(
            books
                .filter(schema::books::dsl::id.eq(book_id))
                .filter(schema::books::dsl::user.eq(auth.0)),
        )
        .set(schema::books::dsl::cover_url.eq(url))
        .execute(connection);
    }

    (StatusCode::OK, Json(json!({ "cover_url": resolved })))
}

#[derive(Debug, Deserialize)]
pub struct RateBookRequest {
    pub book_id: String,
    pub rating: Option<i16>,
}

pub(crate) async fn rate_book(
    auth: AuthUser,
    Json(payload): Json<RateBookRequest>,
) -> impl IntoResponse {
    let book_id = match Uuid::parse_str(&payload.book_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid book ID.".to_string() }))),
    };

    if let Some(r) = payload.rating {
        if !(1..=5).contains(&r) {
            return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Rating must be between 1 and 5.".to_string() })));
        }
    }

    let connection = &mut connect();

    let book: Book = match books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(auth.0))
        .first(connection)
    {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, Json(json!(ErrorResponse { error: "Book not found.".to_string() }))),
    };

    if book.user != auth.0 {
        return (StatusCode::FORBIDDEN, Json(json!(ErrorResponse { error: "Access denied.".to_string() })));
    }

    match diesel::update(
        books.filter(schema::books::dsl::id.eq(book_id)),
    )
    .set(schema::books::dsl::rating.eq(payload.rating))
    .execute(connection)
    {
        Ok(_) => (StatusCode::OK, Json(json!({ "rating": payload.rating }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Failed to update rating: {}", e) }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct SetPageCountRequest {
    pub book_id: String,
    pub page_count: Option<i32>,
}

/// Sets a user-provided page count override for a book. The external catalogs
/// (Google Books / Open Library) often carry wrong or missing page counts; the
/// override is stored on the book row and takes precedence over catalog data.
/// Passing `null` clears the override.
pub(crate) async fn set_page_count(
    auth: AuthUser,
    Json(payload): Json<SetPageCountRequest>,
) -> impl IntoResponse {
    let book_id = match Uuid::parse_str(&payload.book_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Invalid book ID.".to_string() }))),
    };

    if let Some(p) = payload.page_count {
        if p <= 0 {
            return (StatusCode::BAD_REQUEST, Json(json!(ErrorResponse { error: "Page count must be a positive number.".to_string() })));
        }
    }

    let connection = &mut connect();

    match diesel::update(
        books
            .filter(schema::books::dsl::id.eq(book_id))
            .filter(schema::books::dsl::user.eq(auth.0)),
    )
    .set(schema::books::dsl::page_count.eq(payload.page_count))
    .execute(connection)
    {
        Ok(0) => (StatusCode::NOT_FOUND, Json(json!(ErrorResponse { error: "Book not found.".to_string() }))),
        Ok(_) => (StatusCode::OK, Json(json!({ "page_count": payload.page_count }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ErrorResponse { error: format!("Failed to update page count: {}", e) }))),
    }
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

    #[tokio::test]
    async fn test_rate_book_requires_auth() {
        let app = Router::new().route("/api/books/rate", post(rate_book));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/rate").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_resolve_cover_requires_auth() {
        let app = Router::new().route("/api/books/resolve-cover", post(resolve_cover));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/resolve-cover").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_set_page_count_requires_auth() {
        let app = Router::new().route("/api/books/set-page-count", post(set_page_count));
        let response = app
            .oneshot(Request::builder().method("POST").uri("/api/books/set-page-count").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
