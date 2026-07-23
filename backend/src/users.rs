use crate::auth::AuthUser;
use crate::db::connect;
use crate::goodreads_importer::{parse_goodreads_date, BookRecord};
use crate::models::{Reading, ReadingMode, Shelf, User};
use crate::schema::users::dsl::users;
use crate::schema::users::name;
use crate::ErrorResponse;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use uuid::Uuid;

const MAX_IMPORT_BYTES: usize = 5 * 1024 * 1024; // 5 MB

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/user/register", post(register))
        .route("/api/user/login", post(login))
        .route(
            "/api/user/import-good-reads",
            post(import_good_reads).layer(DefaultBodyLimit::max(MAX_IMPORT_BYTES)),
        )
}

/// Request type for registering a new user.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Response type for a successful user registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
}

/// Allows registering a new user.
///
/// This route accepts a JSON payload with the following structure:
/// - `username`: The name of the user to register.
/// - `password`: The password of the user to register.
pub(crate) async fn register(
    result: Result<Json<RegisterRequest>, JsonRejection>,
) -> impl IntoResponse {
    let payload = match result {
        Ok(payload) => payload,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ErrorResponse {
                    error: err.body_text()
                })),
            )
        }
    };

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let new_user = User {
        id: Uuid::new_v4(),
        name: payload.username.clone(),
        password: password_hash,
        elevated: false,
    };

    let connection = &mut connect();

    match diesel::insert_into(users)
        .values(&new_user)
        .execute(connection)
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!(RegisterResponse {
                message: "Successfully registered user.".to_string(),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error while registering the user: {}", e),
            })),
        ),
    }
}

/// Request type for logging in a user.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response type for a successful user login.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
}

/// Attempts to log in a user.
///
/// This route accepts a JSON payload with the following structure:
/// - `username`: The name of the user to log in.
/// - `password`: The password of the user to log in.
pub(crate) async fn login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    let connection = &mut connect();

    let user: User = match users.filter(name.eq(&payload.username)).first(connection) {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!(ErrorResponse {
                    error: "Login failed.".to_string(),
                })),
            )
        }
    };

    // TODO dont use unwrap
    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    let is_valid = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        match crate::auth::create_token(user.id) {
            Ok(token) => (
                StatusCode::OK,
                Json(json!(LoginResponse {
                    token,
                    user_id: user.id.to_string(),
                })),
            ),
            Err(e) => {
                tracing::error!("Failed to create JWT for user {}: {}", user.id, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!(ErrorResponse {
                        error: "Failed to generate authentication token.".to_string(),
                    })),
                )
            }
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ErrorResponse {
                error: "Login failed.".to_string(),
            })),
        )
    }
}

/// Handles importing GoodReads CSV file.
///
/// This route accepts a multipart form data with the following structure:
/// - `file`: The CSV file to import.
///
/// Authentication is required via JWT token in the Authorization header.
pub(crate) async fn import_good_reads(
    auth: AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_uuid = auth.0;
    let mut file_data = None;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Failed to read multipart data: {}", e) })),
                );
            }
        };

        let field_name = match field.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        if field_name == "file" {
            match field.bytes().await {
                Ok(bytes) => file_data = Some(bytes),
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": format!("Failed to read file field: {}", e) })),
                    );
                }
            }
        }
    }

    let Some(file_data) = file_data else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing file." })),
        );
    };

    let records = match BookRecord::from_reader(Cursor::new(file_data)) {
        Ok(records) => records,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Failed to parse CSV file: {}", e) })),
            )
        }
    };

    let connection = &mut connect();

    match users
        .filter(crate::schema::users::dsl::id.eq(user_uuid))
        .first::<User>(connection)
    {
        Ok(_) => {}
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found." })),
            );
        }
    }

    // Collect all unique shelf names referenced in the CSV
    let mut shelf_names: HashSet<String> = HashSet::new();
    for record in &records {
        let exclusive = record.exclusive_shelf.trim().to_string();
        if !exclusive.is_empty() {
            shelf_names.insert(exclusive);
        }
        for shelf in record.bookshelves.split(',') {
            let shelf = shelf.trim().to_string();
            if !shelf.is_empty() {
                shelf_names.insert(shelf);
            }
        }
    }

    // Load existing shelves and build a name -> id map
    let existing_shelves: Vec<Shelf> = match crate::schema::shelves::dsl::shelves
        .filter(crate::schema::shelves::dsl::user.eq(user_uuid))
        .load(connection)
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load shelves: {}", e) })),
            );
        }
    };

    let mut shelf_map: HashMap<String, Uuid> = existing_shelves
        .into_iter()
        .map(|s| (s.name, s.id))
        .collect();

    // Create any shelves from the CSV that don't exist yet
    let now = chrono::Utc::now().naive_utc();
    for shelf_name in &shelf_names {
        if !shelf_map.contains_key(shelf_name) {
            let new_shelf = Shelf {
                id: Uuid::new_v4(),
                name: shelf_name.clone(),
                description: None,
                user: user_uuid,
                created_at: now,
                updated_at: now,
            };
            match diesel::insert_into(crate::schema::shelves::dsl::shelves)
                .values(&new_shelf)
                .execute(connection)
            {
                Ok(_) => {
                    shelf_map.insert(shelf_name.clone(), new_shelf.id);
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            json!({ "error": format!("Failed to create shelf '{}': {}", shelf_name, e) }),
                        ),
                    );
                }
            }
        }
    }

    // Collect unique ISBNs and look them up on Google Books concurrently.
    let unique_isbns: Vec<String> = records
        .iter()
        .filter_map(|r| {
            let isbn13 = r.isbn13.trim_matches(|c| c == '=' || c == '"').to_string();
            let isbn10 = r.isbn.trim_matches(|c| c == '=' || c == '"').to_string();
            if !isbn13.is_empty() {
                Some(isbn13)
            } else if !isbn10.is_empty() {
                Some(isbn10)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let gb_client = reqwest::Client::new();
    use futures::stream::StreamExt;
    let enrichment_map: HashMap<String, String> = futures::stream::iter(unique_isbns)
        .map(|isbn| {
            let client = gb_client.clone();
            async move {
                let id = crate::google_books_client::lookup_id_by_isbn(&client, &isbn).await;
                (isbn, id)
            }
        })
        .buffer_unordered(5)
        .filter_map(|(isbn, id)| async move { id.map(|id| (isbn, id)) })
        .collect()
        .await;

    let mut books_added = 0usize;
    let mut books_skipped = 0usize;
    let mut books_failed = 0usize;
    let mut readings_created = 0usize;

    for record in &records {
        // GoodReads "Date Added" drives the timestamp for the book row and its
        // shelf memberships. Fall back to import time if it is missing/malformed.
        let added_at = parse_goodreads_date(&record.date_added)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .unwrap_or(now);

        // GoodReads exports ISBNs wrapped in ="..." — strip that formatting
        let isbn13 = record
            .isbn13
            .trim_matches(|c| c == '=' || c == '"')
            .to_string();
        let isbn10 = record
            .isbn
            .trim_matches(|c| c == '=' || c == '"')
            .to_string();

        // Collect the target shelves: exclusive shelf + any additional bookshelves
        let mut target_shelves: Vec<String> = vec![record.exclusive_shelf.trim().to_string()];
        for shelf in record.bookshelves.split(',') {
            let shelf = shelf.trim().to_string();
            if !shelf.is_empty() && shelf != record.exclusive_shelf.trim() {
                target_shelves.push(shelf);
            }
        }

        let google_books_id = {
            let isbn = if !isbn13.is_empty() { &isbn13 } else { &isbn10 };
            enrichment_map.get(isbn).cloned()
        };

        let gr_rating = record.my_rating.filter(|r| *r >= 1.0 && *r <= 5.0).map(|r| r as i16);

        let book_id = match crate::books::resolve_or_create_book(
            connection,
            user_uuid,
            Some(record.title.clone()),
            Some(record.author.clone()),
            Some(isbn13.clone()),
            Some(isbn10.clone()),
            google_books_id,
            None,
            added_at,
            gr_rating,
            None,
            record.number_of_pages.map(|p| p as i32),
        ) {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Error resolving book '{}': {}", record.title, e);
                books_failed += 1;
                continue;
            }
        };

        if let Some(r) = gr_rating {
            let _ = diesel::update(
                crate::schema::books::dsl::books.filter(crate::schema::books::dsl::id.eq(book_id)),
            )
            .set(crate::schema::books::dsl::rating.eq(r))
            .execute(connection);
        }

        for shelf_name in &target_shelves {
            if shelf_name.is_empty() {
                continue;
            }
            let shelf_id = match shelf_map.get(shelf_name) {
                Some(&id) => id,
                None => continue,
            };

            let already_member: bool = diesel::select(diesel::dsl::exists(
                crate::schema::book_shelves::dsl::book_shelves
                    .filter(crate::schema::book_shelves::dsl::book.eq(book_id))
                    .filter(crate::schema::book_shelves::dsl::shelf.eq(shelf_id)),
            ))
            .get_result(connection)
            .unwrap_or(false);

            if already_member {
                books_skipped += 1;
                continue;
            }

            match crate::books::ensure_membership(connection, book_id, shelf_id, added_at) {
                Ok(_) => books_added += 1,
                Err(e) => {
                    tracing::error!("Error adding book '{}' to a shelf: {}", record.title, e);
                    books_failed += 1;
                }
            }
        }

        // Derive reading record(s) from the exclusive shelf. The "read" shelf
        // yields finished reading(s); "currently-reading" yields a single open
        // reading so the book surfaces on the home page. Skip if the book
        // already has a reading, so re-importing stays idempotent.
        let exclusive = record.exclusive_shelf.trim();
        if exclusive == "read" || exclusive == "currently-reading" {
            let already_has_reading: bool = diesel::select(diesel::dsl::exists(
                crate::schema::readings::dsl::readings
                    .filter(crate::schema::readings::dsl::book.eq(book_id))
                    .filter(crate::schema::readings::dsl::user.eq(user_uuid)),
            ))
            .get_result(connection)
            .unwrap_or(false);

            if !already_has_reading {
                let total_pages = record.number_of_pages.unwrap_or(0) as i32;
                let added = parse_goodreads_date(&record.date_added);

                // "read" creates one reading per read (Read Count, min 1);
                // "currently-reading" creates a single open reading.
                let count = if exclusive == "read" {
                    record.read_count.max(1)
                } else {
                    1
                };

                for _ in 0..count {
                    let new_reading = if exclusive == "read" {
                        // GoodReads stores a single finish date regardless of read
                        // count; fall back to Date Added, then today, when absent.
                        let finish = record
                            .date_read
                            .as_deref()
                            .and_then(parse_goodreads_date)
                            .or(added)
                            .unwrap_or_else(|| chrono::Utc::now().date_naive());
                        Reading {
                            id: Uuid::new_v4(),
                            book: book_id,
                            user: user_uuid,
                            total_pages,
                            progress: total_pages,
                            mode: ReadingMode::Pages,
                            started_at: finish,
                            finished_at: Some(finish),
                            cancelled_at: None,
                            created_at: now,
                            updated_at: now,
                        }
                    } else {
                        // Open reading: GoodReads exports no progress, so start at 0
                        // with no finish date. Date Added approximates the start.
                        Reading {
                            id: Uuid::new_v4(),
                            book: book_id,
                            user: user_uuid,
                            total_pages,
                            progress: 0,
                            mode: ReadingMode::Pages,
                            started_at: added.unwrap_or_else(|| chrono::Utc::now().date_naive()),
                            finished_at: None,
                            cancelled_at: None,
                            created_at: now,
                            updated_at: now,
                        }
                    };

                    match diesel::insert_into(crate::schema::readings::dsl::readings)
                        .values(&new_reading)
                        .execute(connection)
                    {
                        Ok(_) => readings_created += 1,
                        Err(e) => {
                            tracing::error!(
                                "Error creating reading for book '{}': {}",
                                record.title,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    let message = if books_failed > 0 {
        format!(
            "Import complete. {} books added, {} already present, {} failed to insert, {} readings created.",
            books_added, books_skipped, books_failed, readings_created
        )
    } else {
        format!(
            "Import complete. {} books added, {} already present, {} readings created.",
            books_added, books_skipped, readings_created
        )
    };

    (StatusCode::OK, Json(json!({ "message": message })))
}

#[cfg(test)]
mod tests {
    use super::{import_good_reads, login};
    use axum::{body::Body, http::Request, routing::post, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_login_without_credentials_returns_non_ok() {
        // Without a real DB, empty credentials should return non-200 (likely 500 or 401)
        // This confirms the handler compiles and the route is wired correctly
        let app = Router::new().route("/api/user/login", post(login));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"username":"","password":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_import_good_reads_requires_auth() {
        let app = Router::new().route("/api/user/import-good-reads", post(import_good_reads));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/import-good-reads")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
