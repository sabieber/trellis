use crate::auth::AuthUser;
use crate::db::connect;
use crate::models::{Book, BookLabel, BookShelf, LabelKind, Reading};
use crate::schema::books::dsl::books;
use crate::schema::readings::dsl::readings;
use crate::{schema, ErrorResponse};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// The wire shape of a book, as every handler that returns a list of them sends
/// it — and as the frontend's `ShelfBook` type reads it. One function, so a new
/// field reaches all of those responses at once.
///
/// `noted` is the caller's set of books that carry a note (see
/// `notes::noted_book_ids`), loaded once for the whole list.
pub(crate) fn book_json(book: &Book, noted: &HashSet<Uuid>) -> serde_json::Value {
    json!({
        "id": book.id.to_string(),
        "title": book.title,
        "author": book.author,
        "isbn13": book.isbn13,
        "isbn10": book.isbn10,
        "google_books_id": book.google_books_id,
        "open_library_id": book.open_library_id,
        "added_at": book.added_at.to_string(),
        "rating": book.rating,
        // The proxy URL, never the upstream one — see `covers`. The frontend
        // renders this straight into an `<img src>`, so what it must not be is
        // a slow third-party URL.
        "cover_url": crate::covers::proxy_url(book.id, book.cover_url.as_deref()),
        "page_count": book.page_count,
        "has_notes": noted.contains(&book.id),
    })
}

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
/// `page_count` and the source ids are stored on insert and backfilled onto a
/// matched row where they are still NULL; existing values are never overwritten.
/// A refreshed `page_count` is propagated to the book's readings, which keep
/// their own copy (see `readings::backfill_reading_pages`).
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
    // The caller may be relaying a value the frontend posted back, so this is a
    // trust boundary: only a real catalog URL is allowed to reach the column
    // that `covers::serve_cover` later fetches.
    let cover_url = crate::covers::sanitize_cover_url(cover_url);
    let page_count = page_count.filter(|p| *p > 0);

    let base = || b::books.filter(b::user.eq(user_id)).into_boxed();

    // Returns the matched id, backfilling page_count and the source ids when the
    // row has none. Source ids matter for rows imported before a catalog was
    // consulted: without this they stay NULL forever, since a match found via
    // ISBN/title never writes back what the lookup learned.
    let (gid_backfill, olid_backfill) = (google_books_id.clone(), open_library_id.clone());
    let reuse = move |conn: &mut PgConnection, id: Uuid| -> QueryResult<Uuid> {
        if let Some(ref gid) = gid_backfill {
            diesel::update(
                b::books
                    .filter(b::id.eq(id))
                    .filter(b::google_books_id.is_null()),
            )
            .set(b::google_books_id.eq(gid))
            .execute(conn)?;
        }
        if let Some(ref olid) = olid_backfill {
            diesel::update(
                b::books
                    .filter(b::id.eq(id))
                    .filter(b::open_library_id.is_null()),
            )
            .set(b::open_library_id.eq(olid))
            .execute(conn)?;
        }
        if let Some(p) = page_count {
            diesel::update(
                b::books
                    .filter(b::id.eq(id))
                    .filter(b::page_count.is_null()),
            )
            .set(b::page_count.eq(p))
            .execute(conn)?;
        }
        // Read the count back rather than using the incoming one: the update above
        // never overwrites, so an existing value wins and is what the readings of
        // this book have to be aligned with.
        if let Some(p) = b::books
            .filter(b::id.eq(id))
            .select(b::page_count)
            .first::<Option<i32>>(conn)?
        {
            crate::readings::backfill_reading_pages(conn, id, p)?;
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
    diesel::insert_into(b::books)
        .values(&new_book)
        .execute(conn)?;
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
        .route("/api/books/link-catalog", post(link_catalog))
        .route("/api/books/resolve-cover", post(resolve_cover))
        .route("/api/books/rate", post(rate_book))
        .route("/api/books/set-page-count", post(set_page_count))
        .route("/api/books/add-label", post(add_label))
        .route("/api/books/remove-label", post(remove_label))
        .route("/api/books/label-suggestions", post(suggest_labels))
        .route("/api/authors/books", post(list_author_books))
        .route("/api/books/browse", post(browse_books))
        .route("/api/books/random", post(random_books))
}

/// Request type for getting information about a book.
#[derive(Debug, Deserialize)]
pub struct BookInfoRequest {
    pub book_id: String,
}

/// Response type for book information.
#[derive(Debug, Serialize)]
pub struct BookInfoResponse {
    // The stored row in the wire shape of a search hit. The detail view renders
    // this first and treats the catalog record as enrichment, so a book that no
    // catalog knows still opens.
    pub book: crate::book_search::NormalizedBook,
    pub google_books_id: Option<String>,
    pub open_library_id: Option<String>,
    pub isbn13: Option<String>,
    pub isbn10: Option<String>,
    pub rating: Option<i16>,
    pub cover_url: Option<String>,
    pub page_count: Option<i32>,
    pub readings: Vec<serde_json::Value>,
    pub shelf_ids: Vec<String>,
    // The user's own labels ride along here rather than in a route of their
    // own: this response is already fetched on page load. So do the notes.
    // Keyed by kind slug, one key per kind in `LABEL_KINDS`.
    pub labels: HashMap<&'static str, Vec<String>>,
    pub notes: Vec<serde_json::Value>,
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
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid book ID.".to_string()
                })),
            )
        }
    };

    let db_readings = match readings
        .filter(schema::readings::dsl::book.eq(book_id))
        .filter(schema::readings::dsl::user.eq(auth.0))
        .load::<Reading>(connection)
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error loading readings: {}", e)
                })),
            )
        }
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
        Ok(book) => {
            let labels = labels_for_book(connection, book_id).unwrap_or_default();
            let notes = crate::notes::notes_for_book(connection, book_id).unwrap_or_default();
            (
                StatusCode::OK,
                Json(json!(BookInfoResponse {
                    book: crate::book_search::library_to_normalized(&book),
                    google_books_id: book.google_books_id,
                    open_library_id: book.open_library_id,
                    isbn13: book.isbn13,
                    isbn10: book.isbn10,
                    rating: book.rating,
                    cover_url: crate::covers::proxy_url(book.id, book.cover_url.as_deref()),
                    page_count: book.page_count,
                    readings: json_readings,
                    shelf_ids,
                    labels,
                    notes,
                })),
            )
        }
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

    let client = crate::HTTP.clone();
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

/// Request type for linking a stored book to a catalog record.
#[derive(Debug, Deserialize)]
pub struct LinkCatalogRequest {
    pub book_id: String,
    /// `google` or `openlibrary`.
    pub source: String,
    pub source_id: String,
    pub isbn13: Option<String>,
    pub isbn10: Option<String>,
    pub cover_url: Option<String>,
    pub page_count: Option<i32>,
}

/// Points an existing book row at a catalog record the user picked by hand.
///
/// This is the manual counterpart to `resolve_google_id`: it serves books that
/// were imported before they existed in a catalog, and re-linking a book from
/// one catalog to the other.
///
/// The chosen source id replaces the id of its own kind and *clears the other
/// one*, because the detail view prefers Google over Open Library — a leftover
/// Google id would keep winning after a switch to Open Library.
///
/// ISBNs and the cover follow the chosen edition, but a field the record does
/// not carry keeps its stored value rather than being wiped. `page_count` is
/// only filled when it is still empty: it is editable by hand (see
/// `set_page_count`) and a manual count outranks the catalog's.
///
/// Title and author are never touched. They are the user's own, in the user's
/// own language, and the detail view already shows the catalog's title once a
/// record is linked.
pub(crate) async fn link_catalog(
    auth: AuthUser,
    Json(payload): Json<LinkCatalogRequest>,
) -> impl IntoResponse {
    let connection = &mut connect();

    let bad_request = |message: &str| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse {
                error: message.to_string()
            })),
        )
    };

    let Ok(book_id) = Uuid::parse_str(&payload.book_id) else {
        return bad_request("Invalid book ID.");
    };

    let source_id = match normalize(Some(payload.source_id)) {
        Some(id) => id,
        None => return bad_request("Missing source ID."),
    };

    let (google_books_id, open_library_id) = match payload.source.as_str() {
        "google" => (Some(source_id), None),
        "openlibrary" => (None, Some(source_id)),
        _ => return bad_request("Unknown source."),
    };

    let book = match books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(auth.0))
        .first::<Book>(connection)
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

    let page_count = book.page_count.or(payload.page_count.filter(|p| *p > 0));

    let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::update(
            books
                .filter(schema::books::dsl::id.eq(book_id))
                .filter(schema::books::dsl::user.eq(auth.0)),
        )
        .set((
            schema::books::dsl::google_books_id.eq(&google_books_id),
            schema::books::dsl::open_library_id.eq(&open_library_id),
            schema::books::dsl::isbn13.eq(normalize(payload.isbn13).or(book.isbn13)),
            schema::books::dsl::isbn10.eq(normalize(payload.isbn10).or(book.isbn10)),
            schema::books::dsl::cover_url
                .eq(crate::covers::sanitize_cover_url(payload.cover_url).or(book.cover_url)),
            schema::books::dsl::page_count.eq(page_count),
        ))
        .execute(conn)?;
        if let Some(p) = page_count {
            crate::readings::backfill_reading_pages(conn, book_id, p)?;
        }
        Ok(())
    });

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Book linked to catalog record." })),
        ),
        // The partial unique indexes on (user, google_books_id) and
        // (user, open_library_id) reject a record another of the user's books
        // already claims. That is a duplicate to merge, not something to
        // resolve here.
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        )) => (
            StatusCode::CONFLICT,
            Json(json!(ErrorResponse {
                error: "Another book already uses this catalog record.".to_string()
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error linking the book: {}", e)
            })),
        ),
    }
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
        return (
            StatusCode::OK,
            Json(json!({ "cover_url": crate::covers::proxy_url(book_id, Some(url)) })),
        );
    }

    let client = crate::HTTP.clone();
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
                resolved = nb.base.cover_url;
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

    // Persist the resolved URL (even if None, to avoid re-probing). The
    // upstream URL is what gets stored; the proxy URL is only ever a wire
    // value, derived from it on the way out.
    let resolved = crate::covers::sanitize_cover_url(resolved);
    if let Some(ref url) = resolved {
        let _ = diesel::update(
            books
                .filter(schema::books::dsl::id.eq(book_id))
                .filter(schema::books::dsl::user.eq(auth.0)),
        )
        .set(schema::books::dsl::cover_url.eq(url))
        .execute(connection);
    }

    (
        StatusCode::OK,
        Json(json!({ "cover_url": crate::covers::proxy_url(book_id, resolved.as_deref()) })),
    )
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
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid book ID.".to_string()
                })),
            )
        }
    };

    if let Some(r) = payload.rating {
        if !(1..=5).contains(&r) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Rating must be between 1 and 5.".to_string()
                })),
            );
        }
    }

    let connection = &mut connect();

    let book: Book = match books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(auth.0))
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

    match diesel::update(books.filter(schema::books::dsl::id.eq(book_id)))
        .set(schema::books::dsl::rating.eq(payload.rating))
        .execute(connection)
    {
        Ok(_) => (StatusCode::OK, Json(json!({ "rating": payload.rating }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Failed to update rating: {}", e)
            })),
        ),
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
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Invalid book ID.".to_string()
                })),
            )
        }
    };

    if let Some(p) = payload.page_count {
        if p <= 0 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: "Page count must be a positive number.".to_string()
                })),
            );
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
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "Book not found.".to_string()
            })),
        ),
        Ok(_) => {
            // Readings created before the book knew its page count still carry the
            // 0 sentinel; clearing the override (None) leaves them as they are.
            if let Some(p) = payload.page_count {
                if let Err(e) = crate::readings::backfill_reading_pages(connection, book_id, p) {
                    tracing::error!(
                        "Failed to backfill reading pages for book {}: {}",
                        book_id,
                        e
                    );
                }
            }
            (
                StatusCode::OK,
                Json(json!({ "page_count": payload.page_count })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Failed to update page count: {}", e)
            })),
        ),
    }
}

// Postgres `lower()` on text. Declared here because `diesel::dsl::lower` on the
// postgres backend is the *range* lower(), which does not apply to Text.
diesel::define_sql_function! { fn lower(text: diesel::sql_types::Text) -> diesel::sql_types::Text }

/// Longest label we accept. There is no cap on labels per book: this is
/// single-user-owned data behind auth, and the length cap is what stops a
/// runaway payload.
const MAX_LABEL_LENGTH: usize = 40;

/// Normalizes a user-typed label: trims and collapses runs of internal
/// whitespace, so `" Mystery  Novel "` and `"Mystery Novel"` are one label.
fn normalize_label(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every label kind with the slug the API speaks. New kinds are added here and
/// nowhere else — the handlers below are kind-agnostic.
const LABEL_KINDS: [(LabelKind, &str); 6] = [
    (LabelKind::Genre, "genre"),
    (LabelKind::Tag, "tag"),
    (LabelKind::ReceivedFrom, "received_from"),
    (LabelKind::GivenTo, "given_to"),
    (LabelKind::BorrowedFrom, "borrowed_from"),
    (LabelKind::BorrowedTo, "borrowed_to"),
];

/// The kinds whose labels are people. They share one suggestion pool: the same
/// name is a lender on one book and a recipient on the next, and typing it
/// twice is the thing the suggestions exist to prevent.
const PERSON_KINDS: [LabelKind; 4] = [
    LabelKind::ReceivedFrom,
    LabelKind::GivenTo,
    LabelKind::BorrowedFrom,
    LabelKind::BorrowedTo,
];

fn kind_slug(kind: LabelKind) -> &'static str {
    LABEL_KINDS
        .iter()
        .find(|(candidate, _)| *candidate == kind)
        .map(|(_, slug)| *slug)
        .unwrap_or("tag")
}

/// Request type for adding/removing a label. `kind` is one of `LABEL_KINDS`.
#[derive(Debug, Deserialize)]
pub struct LabelRequest {
    pub book_id: String,
    pub kind: String,
    pub label: String,
}

/// Validates a label request into (book id, kind, normalized label). The `Err`
/// string is the 400 message.
fn parse_label_request(payload: &LabelRequest) -> Result<(Uuid, LabelKind, String), &'static str> {
    let book_id = Uuid::parse_str(&payload.book_id).map_err(|_| "Invalid book ID.")?;
    let kind = LABEL_KINDS
        .iter()
        .find(|(_, slug)| *slug == payload.kind)
        .map(|(kind, _)| *kind)
        .ok_or("Unknown label kind.")?;
    let label = normalize_label(&payload.label);
    if label.is_empty() {
        return Err("Label must not be empty.");
    }
    if label.chars().count() > MAX_LABEL_LENGTH {
        return Err("Label must not be longer than 40 characters.");
    }
    Ok((book_id, kind, label))
}

/// Fans a label row set out into one list per kind, keyed by slug. Every kind
/// gets a key, so the frontend never has to guess between "empty" and "absent".
fn split_by_kind(rows: Vec<(LabelKind, String)>) -> HashMap<&'static str, Vec<String>> {
    let mut split: HashMap<&'static str, Vec<String>> = LABEL_KINDS
        .iter()
        .map(|(_, slug)| (*slug, Vec::new()))
        .collect();
    for (kind, label) in rows {
        split.entry(kind_slug(kind)).or_default().push(label);
    }
    split
}

/// Loads all labels of a book, split by kind.
fn labels_for_book(
    conn: &mut PgConnection,
    book_id: Uuid,
) -> QueryResult<HashMap<&'static str, Vec<String>>> {
    use crate::schema::book_labels::dsl as bl;

    bl::book_labels
        .filter(bl::book.eq(book_id))
        .order(bl::label.asc())
        .select((bl::kind, bl::label))
        .load(conn)
        .map(split_by_kind)
}

/// Resolves the book's owner, ensuring it is the authenticated user. The label
/// row's `user` is taken from here, never from the request payload.
pub(crate) fn owned_book(conn: &mut PgConnection, book_id: Uuid, user_id: Uuid) -> QueryResult<Uuid> {
    books
        .filter(schema::books::dsl::id.eq(book_id))
        .filter(schema::books::dsl::user.eq(user_id))
        .select(schema::books::dsl::user)
        .first(conn)
}

/// Adds a genre or tag to a book. Adding a label the book already carries (in
/// any casing) is a no-op, mirroring `ensure_membership`.
pub(crate) async fn add_label(
    auth: AuthUser,
    Json(payload): Json<LabelRequest>,
) -> impl IntoResponse {
    use crate::schema::book_labels::dsl as bl;

    let (book_id, kind, label) = match parse_label_request(&payload) {
        Ok(parsed) => parsed,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: message.to_string()
                })),
            )
        }
    };

    let connection = &mut connect();

    let owner = match owned_book(connection, book_id, auth.0) {
        Ok(owner) => owner,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!(ErrorResponse {
                    error: "Book not found.".to_string()
                })),
            )
        }
    };

    // Reuse the casing the user already picked for this label, so "mystery"
    // after "Mystery" does not open a second entry in the suggestion list. The
    // case-insensitive unique index is the backstop for two concurrent adds.
    let stored = bl::book_labels
        .filter(bl::user.eq(owner))
        .filter(bl::kind.eq(kind))
        .filter(lower(bl::label).eq(label.to_lowercase()))
        .select(bl::label)
        .first::<String>(connection)
        .optional()
        .ok()
        .flatten()
        .unwrap_or(label);

    let insert = diesel::insert_into(bl::book_labels)
        .values(&BookLabel {
            book: book_id,
            user: owner,
            kind,
            label: stored,
            added_at: chrono::Utc::now().naive_utc(),
        })
        // Untargeted: covers both the primary key and the case-insensitive index.
        .on_conflict_do_nothing()
        .execute(connection);

    if let Err(e) = insert {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Failed to add label: {}", e)
            })),
        );
    }

    label_list_response(connection, book_id, kind)
}

/// Removes a genre or tag from a book. Removing a label that isn't there is a
/// no-op, not an error.
pub(crate) async fn remove_label(
    auth: AuthUser,
    Json(payload): Json<LabelRequest>,
) -> impl IntoResponse {
    use crate::schema::book_labels::dsl as bl;

    let (book_id, kind, label) = match parse_label_request(&payload) {
        Ok(parsed) => parsed,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: message.to_string()
                })),
            )
        }
    };

    let connection = &mut connect();

    if owned_book(connection, book_id, auth.0).is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "Book not found.".to_string()
            })),
        );
    }

    let deleted = diesel::delete(
        bl::book_labels
            .filter(bl::book.eq(book_id))
            .filter(bl::kind.eq(kind))
            .filter(lower(bl::label).eq(label.to_lowercase())),
    )
    .execute(connection);

    if let Err(e) = deleted {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Failed to remove label: {}", e)
            })),
        );
    }

    label_list_response(connection, book_id, kind)
}

/// Shared reply of the write handlers: the book's full label list for the
/// changed kind.
fn label_list_response(
    conn: &mut PgConnection,
    book_id: Uuid,
    kind: LabelKind,
) -> (StatusCode, Json<serde_json::Value>) {
    match labels_for_book(conn, book_id) {
        Ok(mut split) => {
            let labels = split.remove(kind_slug(kind)).unwrap_or_default();
            (StatusCode::OK, Json(json!({ "labels": labels })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error loading labels: {}", e)
            })),
        ),
    }
}

/// Lists every distinct label the user has used, as the three pools the
/// frontend's suggestion dropdowns draw from: `genre`, `tag` and `person` —
/// the last one merged across the four person kinds.
pub(crate) async fn suggest_labels(auth: AuthUser) -> impl IntoResponse {
    use crate::schema::book_labels::dsl as bl;

    let connection = &mut connect();

    match bl::book_labels
        .filter(bl::user.eq(auth.0))
        .select((bl::kind, bl::label))
        .distinct()
        .order(bl::label.asc())
        .load::<(LabelKind, String)>(connection)
    {
        Ok(rows) => {
            // The four person kinds share a pool, so the same name entered as a
            // lender must not come back a second time as a recipient.
            let mut person: Vec<String> = Vec::new();
            for (kind, label) in rows.iter() {
                if PERSON_KINDS.contains(kind)
                    && !person.iter().any(|seen| seen.eq_ignore_ascii_case(label))
                {
                    person.push(label.clone());
                }
            }
            let mut split = split_by_kind(rows);
            (
                StatusCode::OK,
                Json(json!({
                    "genre": split.remove("genre").unwrap_or_default(),
                    "tag": split.remove("tag").unwrap_or_default(),
                    "person": person,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error loading labels: {}", e)
            })),
        ),
    }
}

/// Request type for listing all of a user's books by a given author. The author
/// is a free-text string, matched as described on `books_by_author` (there is no
/// author entity).
#[derive(Debug, Deserialize)]
pub struct AuthorBooksRequest {
    pub author: String,
}

/// Loads the current user's books by one author, newest first.
///
/// The spelling of an author name is not one thing: a GoodReads import writes
/// "J.R.R. Tolkien", the catalogs write "J. R. R. Tolkien", and an author page
/// is reached from both — from a shelf row, which carries the stored string, and
/// from a book detail, which carries the catalog's. So the match ignores case,
/// punctuation and spacing. It also holds a shelf together when the same author
/// arrived under two spellings.
/// ponytail: normalizing in Rust means loading the user's books and filtering
/// them here. A personal library is small; give the column a normalized twin and
/// an index if that ever stops being true.
pub(crate) fn books_by_author(user_id: Uuid, author: &str) -> Vec<Book> {
    let wanted = crate::open_library_client::squashed(author);
    let conn = &mut connect();
    books
        .filter(schema::books::dsl::user.eq(user_id))
        .order(schema::books::dsl::added_at.desc())
        .select(Book::as_select())
        .load::<Book>(conn)
        .unwrap_or_default()
        .into_iter()
        .filter(|book| {
            book.author
                .as_deref()
                .is_some_and(|name| crate::open_library_client::squashed(name) == wanted)
        })
        .collect()
}

/// Lists the current user's books by the requested author.
pub(crate) async fn list_author_books(
    auth: AuthUser,
    Json(payload): Json<AuthorBooksRequest>,
) -> impl IntoResponse {
    let results = books_by_author(auth.0, &payload.author);
    let noted = crate::notes::noted_book_ids(&mut connect(), auth.0);
    let json_books: Vec<_> = results.iter().map(|b| book_json(b, &noted)).collect();

    (
        StatusCode::OK,
        Json(json!({
            "author": payload.author,
            "books": json_books,
        })),
    )
}

/// Filters for the library browse view. Every field is optional and an absent
/// (or empty) field does not constrain the result, so the same handler serves
/// the unfiltered landing state. `offset` pages through the result.
#[derive(Debug, Deserialize)]
pub struct BrowseRequest {
    pub shelf_id: Option<String>,
    pub author: Option<String>,
    pub genre: Option<String>,
    pub tag: Option<String>,
    pub rating: Option<String>,
    pub offset: Option<i64>,
}

/// Books per browse page. The client appends the next page on demand and the
/// response carries the total, so it knows when to stop asking.
const BROWSE_PAGE_SIZE: i64 = 100;

/// Sentinel genre/tag/rating filter value meaning "nothing set at all". It rides
/// in the same field as a real value so the whole filter path — URL state,
/// browse, random picker — stays one mechanism.
/// ponytail: a user who names a label exactly this string collides with it;
/// give the filter its own boolean field if that ever happens.
pub(crate) const UNLABELLED: &str = "__none__";

/// The filters of a browse request, parsed and validated.
struct BrowseFilters {
    user_id: Uuid,
    shelf_id: Option<Uuid>,
    author: Option<String>,
    genre: Option<String>,
    tag: Option<String>,
    /// `None` does not filter at all.
    rating: Option<RatingFilter>,
}

/// What a rating filter selects. A star reader picks an exact score, a thumbs
/// reader a tendency — both end up as an inclusive range over the same column.
enum RatingFilter {
    /// No rating set at all.
    Unrated,
    /// Inclusive range: an exact star is `(n, n)`, thumbs up `(4, 5)`, thumbs
    /// down `(1, 2)`. A 3 has no tendency, so neither thumb matches it.
    Range(i16, i16),
}

/// Builds the filtered book query. Called once for the count and once for the
/// page, so both always apply exactly the same filters.
fn browse_query<'a>(
    filters: &BrowseFilters,
) -> crate::schema::books::BoxedQuery<'a, diesel::pg::Pg> {
    use crate::schema::book_labels::dsl as bl;
    use crate::schema::book_shelves::dsl as bs;
    use crate::schema::books::dsl as b;

    let mut query = b::books.filter(b::user.eq(filters.user_id)).into_boxed();

    if let Some(shelf_id) = filters.shelf_id {
        // No ownership check on the shelf: the outer filter already restricts
        // the result to the caller's own books.
        query = query.filter(
            b::id.eq_any(
                bs::book_shelves
                    .filter(bs::shelf.eq(shelf_id))
                    .select(bs::book),
            ),
        );
    }

    if let Some(ref author) = filters.author {
        query = query.filter(b::author.eq(author.clone()));
    }

    query = match filters.rating {
        Some(RatingFilter::Range(low, high)) => query.filter(b::rating.between(low, high)),
        Some(RatingFilter::Unrated) => query.filter(b::rating.is_null()),
        None => query,
    };

    for (kind, value) in [
        (LabelKind::Genre, &filters.genre),
        (LabelKind::Tag, &filters.tag),
    ] {
        let Some(label) = value else { continue };

        // `user` and `kind` first: they are the leading columns of
        // book_labels_user_kind_label_idx, so the subquery is a range scan of
        // this user's labels instead of a scan of the table.
        let labelled = bl::book_labels
            .filter(bl::user.eq(filters.user_id))
            .filter(bl::kind.eq(kind))
            .into_boxed();

        query = if label == UNLABELLED {
            // "no genre / tag set yet": every book without a label of this kind.
            query.filter(b::id.ne_all(labelled.select(bl::book)))
        } else {
            query.filter(
                b::id.eq_any(
                    labelled
                        .filter(lower(bl::label).eq(label.to_lowercase()))
                        .select(bl::book),
                ),
            )
        };
    }

    query
}

/// Validates a browse request into the filters both browse handlers run on. The
/// `Err` string is the 400 message.
fn parse_browse_filters(
    user_id: Uuid,
    payload: &BrowseRequest,
) -> Result<BrowseFilters, &'static str> {
    // `normalize` maps a blank string to None, which is what the frontend's
    // "all" option submits.
    let shelf_id = match normalize(payload.shelf_id.clone()) {
        Some(raw) => Some(Uuid::parse_str(&raw).map_err(|_| "Invalid shelf ID.")?),
        None => None,
    };

    // The column only ever holds 1..=5, so anything else is a bad request rather
    // than an empty result. `up`/`down` come from the thumbs mode of the filter.
    let rating = match normalize(payload.rating.clone()) {
        Some(raw) if raw == UNLABELLED => Some(RatingFilter::Unrated),
        Some(raw) if raw == "up" => Some(RatingFilter::Range(4, 5)),
        Some(raw) if raw == "down" => Some(RatingFilter::Range(1, 2)),
        Some(raw) => match raw.parse::<i16>() {
            Ok(value) if (1..=5).contains(&value) => Some(RatingFilter::Range(value, value)),
            _ => return Err("Invalid rating."),
        },
        None => None,
    };

    Ok(BrowseFilters {
        user_id,
        shelf_id,
        author: normalize(payload.author.clone()),
        genre: normalize(payload.genre.clone()),
        tag: normalize(payload.tag.clone()),
        rating,
    })
}

/// Lists one page of the user's books matching the given filters, plus the
/// author list the filter bar offers. Filtering and paging happen here, never
/// in the client: the view must never assume it holds the whole library.
pub(crate) async fn browse_books(
    auth: AuthUser,
    Json(payload): Json<BrowseRequest>,
) -> impl IntoResponse {
    use crate::schema::books::dsl as b;

    let filters = match parse_browse_filters(auth.0, &payload) {
        Ok(filters) => filters,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: message.to_string()
                })),
            )
        }
    };

    // A negative offset would make postgres error out; treat it as the first page.
    let offset = payload.offset.unwrap_or(0).max(0);

    let connection = &mut connect();

    let total: i64 = match browse_query(&filters).count().get_result(connection) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error counting books: {}", e)
                })),
            )
        }
    };

    // `id` breaks ties so that paging never skips or repeats a book whose title
    // another book shares — `title` alone is not a unique ordering.
    let results = match browse_query(&filters)
        .order((b::title.asc(), b::id.asc()))
        .offset(offset)
        .limit(BROWSE_PAGE_SIZE)
        .select(Book::as_select())
        .load::<Book>(connection)
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error loading books: {}", e)
                })),
            )
        }
    };

    // Only with the first page: the list cannot change while the user pages
    // through a result set, so repeating it on every "load more" is pure waste.
    // Unfiltered on purpose — the author dropdown must keep offering every author
    // of the library, including the one currently selected.
    let authors: Vec<String> = if offset == 0 {
        match b::books
            .filter(b::user.eq(auth.0))
            .select(b::author)
            .distinct()
            .order(b::author.asc())
            .load::<Option<String>>(connection)
        {
            Ok(rows) => rows.into_iter().flatten().collect(),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!(ErrorResponse {
                        error: format!("Error loading authors: {}", e)
                    })),
                )
            }
        }
    } else {
        Vec::new()
    };

    let noted = crate::notes::noted_book_ids(connection, auth.0);
    let json_books: Vec<_> = results.iter().map(|b| book_json(b, &noted)).collect();

    (
        StatusCode::OK,
        Json(json!({
            "books": json_books,
            "authors": authors,
            "total": total,
        })),
    )
}

// Postgres `random()`. Ordering by it is a sort of the whole filtered set, which
// is fine at library sizes; revisit with a sampling scheme if it ever is not.
diesel::define_sql_function! { fn random() -> diesel::sql_types::Double }

/// Books handed to the random picker. It builds one board of spines around the
/// drawn book and redraws from the same handful, so it never needs the whole set.
const RANDOM_CANDIDATES: i64 = 40;

/// Draws a random handful of books from the *filtered* set. The client cannot do
/// this itself: it holds only the pages it has loaded, so picking there would
/// never reach a book further down the alphabet.
pub(crate) async fn random_books(
    auth: AuthUser,
    Json(payload): Json<BrowseRequest>,
) -> impl IntoResponse {
    let filters = match parse_browse_filters(auth.0, &payload) {
        Ok(filters) => filters,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: message.to_string()
                })),
            )
        }
    };

    let connection = &mut connect();

    let results = match browse_query(&filters)
        .order(random())
        .limit(RANDOM_CANDIDATES)
        .select(Book::as_select())
        .load::<Book>(connection)
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ErrorResponse {
                    error: format!("Error loading books: {}", e)
                })),
            )
        }
    };

    let noted = crate::notes::noted_book_ids(connection, auth.0);
    let json_books: Vec<_> = results.iter().map(|b| book_json(b, &noted)).collect();

    (StatusCode::OK, Json(json!({ "books": json_books })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::post, Router};
    use tower::ServiceExt;

    /// The thumbs filter has to reach the same rows as the star filter it
    /// replaces: `up` is 4..5, `down` is 1..2, and a 3 is in neither.
    #[test]
    fn test_rating_filter_parsing() {
        let parse = |raw: &str| {
            let payload = BrowseRequest {
                shelf_id: None,
                author: None,
                genre: None,
                tag: None,
                rating: Some(raw.to_string()),
                offset: None,
            };
            parse_browse_filters(Uuid::nil(), &payload).map(|f| match f.rating {
                Some(RatingFilter::Range(low, high)) => format!("{low}..{high}"),
                Some(RatingFilter::Unrated) => "unrated".to_string(),
                None => "all".to_string(),
            })
        };

        assert_eq!(parse("up").unwrap(), "4..5");
        assert_eq!(parse("down").unwrap(), "1..2");
        assert_eq!(parse("3").unwrap(), "3..3");
        assert_eq!(parse(UNLABELLED).unwrap(), "unrated");
        assert_eq!(parse("").unwrap(), "all");
        assert!(parse("6").is_err());
        assert!(parse("sideways").is_err());
    }

    /// Every kind survives the round trip through its slug, and the split keeps
    /// a key for a kind with no labels on it.
    #[test]
    fn test_label_kinds_round_trip() {
        for (kind, slug) in LABEL_KINDS {
            let payload = LabelRequest {
                book_id: Uuid::nil().to_string(),
                kind: slug.to_string(),
                label: "  Aunt   Mary ".to_string(),
            };
            let (_, parsed, label) = parse_label_request(&payload).unwrap();
            assert_eq!(parsed, kind);
            assert_eq!(kind_slug(parsed), slug);
            assert_eq!(label, "Aunt Mary");
        }

        let split = split_by_kind(vec![(LabelKind::GivenTo, "Mary".to_string())]);
        assert_eq!(split.len(), LABEL_KINDS.len());
        assert_eq!(split["given_to"], vec!["Mary".to_string()]);
        assert!(split["genre"].is_empty());
    }

    #[tokio::test]
    async fn test_get_book_info_requires_auth() {
        let app = Router::new().route("/api/books/info", post(get_book_info));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/info")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_resolve_google_id_requires_auth() {
        let app = Router::new().route("/api/books/resolve-google-id", post(resolve_google_id));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/resolve-google-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_link_catalog_requires_auth() {
        let app = Router::new().route("/api/books/link-catalog", post(link_catalog));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/link-catalog")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_rate_book_requires_auth() {
        let app = Router::new().route("/api/books/rate", post(rate_book));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/rate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_resolve_cover_requires_auth() {
        let app = Router::new().route("/api/books/resolve-cover", post(resolve_cover));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/resolve-cover")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_set_page_count_requires_auth() {
        let app = Router::new().route("/api/books/set-page-count", post(set_page_count));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/set-page-count")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_add_label_requires_auth() {
        let app = Router::new().route("/api/books/add-label", post(add_label));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/add-label")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_remove_label_requires_auth() {
        let app = Router::new().route("/api/books/remove-label", post(remove_label));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/remove-label")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_browse_books_requires_auth() {
        let app = Router::new().route("/api/books/browse", post(browse_books));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/browse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_random_books_requires_auth() {
        let app = Router::new().route("/api/books/random", post(random_books));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/random")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_suggest_labels_requires_auth() {
        let app = Router::new().route("/api/books/label-suggestions", post(suggest_labels));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/label-suggestions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    /// The one piece of non-trivial logic in the labels feature: normalization,
    /// and that a differently-cased re-entry folds onto the stored spelling
    /// (which is what the `lower(label) = lower(?)` lookup in `add_label` keys
    /// on) instead of creating a second suggestion.
    #[test]
    fn test_label_normalization_and_case_folding() {
        assert_eq!(normalize_label(" Mystery  Novel "), "Mystery Novel");
        assert_eq!(normalize_label("\tSci-Fi\n"), "Sci-Fi");
        assert_eq!(normalize_label("   "), "");

        let stored = normalize_label(" Mystery  Novel ");
        let retyped = normalize_label("mystery novel");
        assert_ne!(stored, retyped);
        assert_eq!(stored.to_lowercase(), retyped.to_lowercase());
    }

    #[test]
    fn test_parse_label_request_rejects_bad_input() {
        let request = |kind: &str, label: &str| LabelRequest {
            book_id: Uuid::new_v4().to_string(),
            kind: kind.to_string(),
            label: label.to_string(),
        };

        assert!(parse_label_request(&request("genre", "  ")).is_err());
        assert!(parse_label_request(&request("shelf", "Mystery")).is_err());
        assert!(parse_label_request(&request("genre", &"x".repeat(41))).is_err());
        assert!(parse_label_request(&request("tag", &"x".repeat(40))).is_ok());
        assert!(parse_label_request(&LabelRequest {
            book_id: "not-a-uuid".to_string(),
            kind: "genre".to_string(),
            label: "Mystery".to_string(),
        })
        .is_err());
    }

    #[tokio::test]
    async fn test_list_author_books_requires_auth() {
        let app = Router::new().route("/api/authors/books", post(list_author_books));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/authors/books")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
