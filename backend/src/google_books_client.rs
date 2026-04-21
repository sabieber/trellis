use crate::auth::AuthUser;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/google-books/search", get(search_books))
        .route("/api/google-books/volume/{id}", get(get_volume))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn lookup_id_by_isbn(client: &Client, isbn: &str) -> Option<String> {
    if isbn.is_empty() {
        return None;
    }
    let mut req = client
        .get("https://www.googleapis.com/books/v1/volumes")
        .query(&[("q", &format!("isbn:{}", isbn)), ("maxResults", &"1".to_string())]);
    if let Some(key) = std::env::var("GOOGLE_BOOKS_API_KEY").ok() {
        req = req.query(&[("key", key)]);
    }
    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    json["items"][0]["id"].as_str().map(|s| s.to_string())
}

pub(crate) async fn search_books(
    _auth: AuthUser,
    Query(params): Query<SearchQuery>,
) -> Response {
    let client = Client::new();
    let mut req = client
        .get("https://www.googleapis.com/books/v1/volumes")
        .query(&[("q", params.q.as_str())]);
    if let Some(key) = std::env::var("GOOGLE_BOOKS_API_KEY").ok() {
        req = req.query(&[("key", key.as_str())]);
    }
    proxy_request(req).await
}

pub(crate) async fn get_volume(
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Response {
    let client = Client::new();
    let mut req = client.get(format!(
        "https://www.googleapis.com/books/v1/volumes/{}",
        id
    ));
    if let Some(key) = std::env::var("GOOGLE_BOOKS_API_KEY").ok() {
        req = req.query(&[("key", key.as_str())]);
    }
    proxy_request(req).await
}

async fn proxy_request(req: reqwest::RequestBuilder) -> Response {
    match req.send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
            Ok(body) => (StatusCode::OK, Json(body)).into_response(),
            Err(_) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "Invalid response from Google Books" })),
            )
                .into_response(),
        },
        Ok(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "Google Books request failed" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "Failed to reach Google Books" })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn empty_isbn_returns_none() {
        let client = Client::new();
        let result = lookup_id_by_isbn(&client, "").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn known_isbn_returns_google_books_id() {
        let client = Client::new();
        let result = lookup_id_by_isbn(&client, "9780135957059").await;
        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn search_requires_auth() {
        let app = Router::new().route("/api/google-books/search", get(search_books));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/google-books/search?q=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn volume_requires_auth() {
        let app = Router::new().route("/api/google-books/volume/{id}", get(get_volume));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/google-books/volume/some-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
