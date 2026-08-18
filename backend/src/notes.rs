use crate::auth::AuthUser;
use crate::books::owned_book;
use crate::db::connect;
use crate::models::BookNote;
use crate::schema::book_notes::dsl as bn;
use crate::ErrorResponse;
use axum::routing::post;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, Router};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

type ApiResponse = (StatusCode, Json<serde_json::Value>);

/// Longest note we accept. Long enough for a page of typed thoughts, short
/// enough that a runaway paste does not land in the database.
const MAX_NOTE_LENGTH: usize = 10_000;

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/notes/save", post(save_note))
        .route("/api/books/notes/delete", post(delete_note))
}

/// Create (no `note_id`) and edit (with one) are the same request: the fields
/// the user can set are identical, so one handler covers both.
#[derive(Debug, Deserialize)]
pub struct SaveNoteRequest {
    pub book_id: String,
    pub note_id: Option<String>,
    pub text: String,
    pub page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteNoteRequest {
    pub note_id: String,
}

fn bad_request(message: &str) -> ApiResponse {
    (
        StatusCode::BAD_REQUEST,
        Json(json!(ErrorResponse {
            error: message.to_string()
        })),
    )
}

fn not_found(message: &str) -> ApiResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!(ErrorResponse {
            error: message.to_string()
        })),
    )
}

/// Loads a book's notes, newest first, in the shape the detail view renders.
/// The caller has already checked that the book belongs to the user.
pub(crate) fn notes_for_book(
    conn: &mut PgConnection,
    book_id: Uuid,
) -> QueryResult<Vec<serde_json::Value>> {
    let rows = bn::book_notes
        .filter(bn::book.eq(book_id))
        .order(bn::created_at.desc())
        .load::<BookNote>(conn)?;

    Ok(rows
        .into_iter()
        .map(|note| {
            json!({
                "id": note.id.to_string(),
                "text": note.text,
                "page": note.page,
                "created_at": note.created_at.and_utc().to_rfc3339(),
                "updated_at": note.updated_at.and_utc().to_rfc3339(),
            })
        })
        .collect())
}

/// Every book of this user that carries at least one note — what the note badge
/// on a cover is drawn from. Loaded once per list request, so the badge costs
/// one query for the whole list rather than one per book.
pub(crate) fn noted_book_ids(conn: &mut PgConnection, user_id: Uuid) -> HashSet<Uuid> {
    bn::book_notes
        .filter(bn::user.eq(user_id))
        .select(bn::book)
        .distinct()
        .load::<Uuid>(conn)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

/// Both write handlers answer with the book's full note list, so the caller
/// replaces its list instead of patching it.
fn note_list_response(conn: &mut PgConnection, book_id: Uuid) -> ApiResponse {
    match notes_for_book(conn, book_id) {
        Ok(notes) => (StatusCode::OK, Json(json!({ "notes": notes }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error loading notes: {}", e)
            })),
        ),
    }
}

/// Creates a note, or overwrites the text and page of an existing one. The
/// stored `created_at` and `book` never change on an edit.
pub(crate) async fn save_note(
    auth: AuthUser,
    Json(payload): Json<SaveNoteRequest>,
) -> impl IntoResponse {
    let Ok(book_id) = Uuid::parse_str(&payload.book_id) else {
        return bad_request("Invalid book ID.");
    };

    let text = payload.text.trim();
    if text.is_empty() {
        return bad_request("Note must not be empty.");
    }
    if text.chars().count() > MAX_NOTE_LENGTH {
        return bad_request("Note is too long.");
    }
    // A page of 0 or less is a typo, not "no page" — the frontend omits the
    // field for that.
    if payload.page.is_some_and(|page| page <= 0) {
        return bad_request("Page must be a positive number.");
    }

    let connection = &mut connect();

    // The note's `user` comes from the book row, never from the payload.
    let Ok(owner) = owned_book(connection, book_id, auth.0) else {
        return not_found("Book not found.");
    };

    let now = chrono::Utc::now().naive_utc();

    let result = match payload.note_id {
        Some(ref id) => {
            let Ok(note_id) = Uuid::parse_str(id) else {
                return bad_request("Invalid note ID.");
            };
            let updated = diesel::update(
                bn::book_notes
                    .filter(bn::id.eq(note_id))
                    .filter(bn::user.eq(auth.0)),
            )
            .set((
                bn::text.eq(text),
                bn::page.eq(payload.page),
                bn::updated_at.eq(now),
            ))
            .execute(connection);
            match updated {
                // Zero rows means the note is gone or belongs to someone else;
                // either way there is nothing here for this user to edit.
                Ok(0) => return not_found("Note not found."),
                other => other,
            }
        }
        None => diesel::insert_into(bn::book_notes)
            .values(&BookNote {
                id: Uuid::new_v4(),
                book: book_id,
                user: owner,
                text: text.to_string(),
                page: payload.page,
                created_at: now,
                updated_at: now,
            })
            .execute(connection),
    };

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error saving note: {}", e)
            })),
        );
    }

    note_list_response(connection, book_id)
}

pub(crate) async fn delete_note(
    auth: AuthUser,
    Json(payload): Json<DeleteNoteRequest>,
) -> impl IntoResponse {
    let Ok(note_id) = Uuid::parse_str(&payload.note_id) else {
        return bad_request("Invalid note ID.");
    };

    let connection = &mut connect();

    // Loaded first for its book id, which the reply's note list needs.
    let note = bn::book_notes
        .filter(bn::id.eq(note_id))
        .filter(bn::user.eq(auth.0))
        .first::<BookNote>(connection);

    let Ok(note) = note else {
        return not_found("Note not found.");
    };

    match diesel::delete(bn::book_notes.filter(bn::id.eq(note_id))).execute(connection) {
        Ok(_) => note_list_response(connection, note.book),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: format!("Error deleting note: {}", e)
            })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_save_note_requires_auth() {
        let app = Router::new().route("/api/books/notes/save", post(save_note));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/notes/save")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_delete_note_requires_auth() {
        let app = Router::new().route("/api/books/notes/delete", post(delete_note));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/books/notes/delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
