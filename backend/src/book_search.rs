use crate::auth::AuthUser;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedBook {
    pub id: String,
    pub source: String,
    pub source_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub cover_url: Option<String>,
    pub published_year: Option<String>,
    pub page_count: Option<u32>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub average_rating: Option<f64>,
    pub isbn13: Option<String>,
    pub isbn10: Option<String>,
}

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/search", get(unified_search))
        .route("/api/books/external/{source}/{id}", get(external_detail))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

pub(crate) async fn unified_search(
    _auth: AuthUser,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let client = Client::new();
    let query = params.query.clone();

    let google_future = google_search(client.clone(), query.clone());
    let ol_future = crate::open_library_client::search(&client, &query);

    let (google_results, ol_results) = tokio::join!(google_future, ol_future);

    let merged = merge_results(google_results, ol_results);

    (StatusCode::OK, Json(json!(merged)))
}

pub(crate) async fn external_detail(
    _auth: AuthUser,
    Path((source, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let client = Client::new();

    let result = match source.as_str() {
        "google" => google_detail(&client, &id).await,
        "openlibrary" => crate::open_library_client::get_work(&client, &id).await,
        _ => None,
    };

    match result {
        Some(book) => (StatusCode::OK, Json(json!(book))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Book not found" })),
        )
            .into_response(),
    }
}

async fn google_search(client: Client, query: String) -> Vec<NormalizedBook> {
    let mut request = client
        .get("https://www.googleapis.com/books/v1/volumes")
        .query(&[("q", query.as_str())]);
    if let Ok(key) = std::env::var("GOOGLE_BOOKS_API_KEY") {
        request = request.query(&[("key", key.as_str())]);
    }
    let response = match request.send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return vec![],
    };
    let body: Value = match response.json().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };
    let items = match body["items"].as_array() {
        Some(i) => i,
        None => return vec![],
    };
    items.iter().filter_map(normalize_google_item).collect()
}

async fn google_detail(client: &Client, volume_id: &str) -> Option<NormalizedBook> {
    let mut request = client.get(format!(
        "https://www.googleapis.com/books/v1/volumes/{}",
        volume_id
    ));
    if let Ok(key) = std::env::var("GOOGLE_BOOKS_API_KEY") {
        request = request.query(&[("key", key.as_str())]);
    }
    let response = match request.send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };
    let body: Value = match response.json().await {
        Ok(b) => b,
        Err(_) => return None,
    };
    normalize_google_item(&body)
}

fn normalize_google_item(item: &Value) -> Option<NormalizedBook> {
    let id = item["id"].as_str()?.to_string();
    let volume_info = &item["volumeInfo"];
    let title = volume_info["title"].as_str()?.to_string();

    let authors: Vec<String> = volume_info["authors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let cover_url = volume_info["imageLinks"]["thumbnail"]
        .as_str()
        .map(String::from);

    let published_year = volume_info["publishedDate"]
        .as_str()
        .and_then(|d| d.get(..4))
        .map(String::from);

    let page_count = volume_info["pageCount"].as_u64().map(|p| p as u32);

    let category = volume_info["categories"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    let description = volume_info["description"].as_str().map(String::from);

    let average_rating = volume_info["averageRating"].as_f64();

    let (isbn13, isbn10) = extract_google_isbns(volume_info);

    Some(NormalizedBook {
        id: format!("google:{}", id),
        source: "google".to_string(),
        source_id: id,
        title,
        authors,
        cover_url,
        published_year,
        page_count,
        category,
        description,
        average_rating,
        isbn13,
        isbn10,
    })
}

fn extract_google_isbns(volume_info: &Value) -> (Option<String>, Option<String>) {
    let mut isbn13 = None;
    let mut isbn10 = None;
    if let Some(ids) = volume_info["industryIdentifiers"].as_array() {
        for id in ids {
            match id["type"].as_str() {
                Some("ISBN_13") => isbn13 = id["identifier"].as_str().map(String::from),
                Some("ISBN_10") => isbn10 = id["identifier"].as_str().map(String::from),
                _ => {}
            }
        }
    }
    (isbn13, isbn10)
}

fn isbn_key(book: &NormalizedBook) -> Option<String> {
    book.isbn13.clone().or_else(|| book.isbn10.clone())
}

fn merge_results(google: Vec<NormalizedBook>, ol: Vec<NormalizedBook>) -> Vec<NormalizedBook> {
    let mut results: Vec<NormalizedBook> = google;

    let mut google_isbn_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, book) in results.iter().enumerate() {
        if let Some(key) = isbn_key(book) {
            google_isbn_to_idx.entry(key).or_insert(i);
        }
    }
    let mut enriched: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for book in ol {
        if let Some(key) = isbn_key(&book) {
            if let Some(&idx) = google_isbn_to_idx.get(&key) {
                if enriched.insert(idx) {
                    enrich_from_ol(&mut results[idx], &book);
                    continue;
                }
            }
        }
        results.push(book);
    }

    results
}

fn enrich_from_ol(target: &mut NormalizedBook, ol: &NormalizedBook) {
    if target.cover_url.is_none() && ol.cover_url.is_some() {
        target.cover_url = ol.cover_url.clone();
    }
    if target.description.is_none() && ol.description.is_some() {
        target.description = ol.description.clone();
    }
    if target.page_count.is_none() && ol.page_count.is_some() {
        target.page_count = ol.page_count;
    }
    if target.category.is_none() && ol.category.is_some() {
        target.category = ol.category.clone();
    }
    if target.isbn13.is_none() && ol.isbn13.is_some() {
        target.isbn13 = ol.isbn13.clone();
    }
    if target.isbn10.is_none() && ol.isbn10.is_some() {
        target.isbn10 = ol.isbn10.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use tower::ServiceExt;

    #[test]
    fn merge_deduplicates_by_isbn() {
        let google = vec![NormalizedBook {
            id: "google:abc".into(),
            source: "google".into(),
            source_id: "abc".into(),
            title: "Test Book".into(),
            authors: vec!["Author".into()],
            cover_url: Some("http://google.com/cover.jpg".into()),
            published_year: Some("2020".into()),
            page_count: Some(200),
            category: Some("Fiction".into()),
            description: Some("A test book.".into()),
            average_rating: Some(4.5),
            isbn13: Some("9781234567890".into()),
            isbn10: None,
        }];

        let ol = vec![NormalizedBook {
            id: "openlibrary:/works/OL123W".into(),
            source: "openlibrary".into(),
            source_id: "/works/OL123W".into(),
            title: "Test Book".into(),
            authors: vec!["Author".into()],
            cover_url: None,
            published_year: Some("2020".into()),
            page_count: None,
            category: None,
            description: None,
            average_rating: None,
            isbn13: Some("9781234567890".into()),
            isbn10: None,
        }];

        let merged = merge_results(google, ol);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source, "google");
        assert_eq!(merged[0].isbn13, Some("9781234567890".into()));
    }

    #[test]
    fn merge_keeps_unique_books() {
        let google = vec![NormalizedBook {
            id: "google:abc".into(),
            source: "google".into(),
            source_id: "abc".into(),
            title: "Book A".into(),
            authors: vec![],
            cover_url: None,
            published_year: None,
            page_count: None,
            category: None,
            description: None,
            average_rating: None,
            isbn13: Some("1111111111111".into()),
            isbn10: None,
        }];

        let ol = vec![NormalizedBook {
            id: "openlibrary:/works/OL456W".into(),
            source: "openlibrary".into(),
            source_id: "/works/OL456W".into(),
            title: "Book B".into(),
            authors: vec![],
            cover_url: None,
            published_year: None,
            page_count: None,
            category: None,
            description: None,
            average_rating: None,
            isbn13: Some("2222222222222".into()),
            isbn10: None,
        }];

        let merged = merge_results(google, ol);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn merge_keeps_same_source_books_with_shared_isbn() {
        let google = vec![];
        let ol = vec![
            NormalizedBook {
                id: "openlibrary:/works/OL111W".into(),
                source: "openlibrary".into(),
                source_id: "/works/OL111W".into(),
                title: "Book Part 1".into(),
                authors: vec!["Author".into()],
                cover_url: None,
                published_year: None,
                page_count: None,
                category: None,
                description: None,
                average_rating: None,
                isbn13: Some("9781234567890".into()),
                isbn10: None,
            },
            NormalizedBook {
                id: "openlibrary:/works/OL222W".into(),
                source: "openlibrary".into(),
                source_id: "/works/OL222W".into(),
                title: "Book Part 2".into(),
                authors: vec!["Author".into()],
                cover_url: None,
                published_year: None,
                page_count: None,
                category: None,
                description: None,
                average_rating: None,
                isbn13: Some("9781234567890".into()),
                isbn10: None,
            },
            NormalizedBook {
                id: "openlibrary:/works/OL333W".into(),
                source: "openlibrary".into(),
                source_id: "/works/OL333W".into(),
                title: "Book Part 3".into(),
                authors: vec!["Author".into()],
                cover_url: None,
                published_year: None,
                page_count: None,
                category: None,
                description: None,
                average_rating: None,
                isbn13: Some("9781234567890".into()),
                isbn10: None,
            },
        ];

        let merged = merge_results(google, ol);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].title, "Book Part 1");
        assert_eq!(merged[1].title, "Book Part 2");
        assert_eq!(merged[2].title, "Book Part 3");
    }

    #[test]
    fn merge_enriches_first_same_source_book_only() {
        let google = vec![NormalizedBook {
            id: "google:abc".into(),
            source: "google".into(),
            source_id: "abc".into(),
            title: "Book Part 1".into(),
            authors: vec!["Author".into()],
            cover_url: None,
            published_year: None,
            page_count: None,
            category: None,
            description: None,
            average_rating: None,
            isbn13: Some("9781234567890".into()),
            isbn10: None,
        }];

        let ol = vec![
            NormalizedBook {
                id: "openlibrary:/works/OL111W".into(),
                source: "openlibrary".into(),
                source_id: "/works/OL111W".into(),
                title: "Book Part 1".into(),
                authors: vec![],
                cover_url: Some("http://ol.com/cover.jpg".into()),
                published_year: None,
                page_count: None,
                category: None,
                description: None,
                average_rating: None,
                isbn13: Some("9781234567890".into()),
                isbn10: None,
            },
            NormalizedBook {
                id: "openlibrary:/works/OL222W".into(),
                source: "openlibrary".into(),
                source_id: "/works/OL222W".into(),
                title: "Book Part 2".into(),
                authors: vec!["Author".into()],
                cover_url: None,
                published_year: None,
                page_count: None,
                category: None,
                description: None,
                average_rating: None,
                isbn13: Some("9781234567890".into()),
                isbn10: None,
            },
        ];

        let merged = merge_results(google, ol);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].source, "google");
        assert_eq!(merged[0].cover_url, Some("http://ol.com/cover.jpg".into()));
        assert_eq!(merged[1].title, "Book Part 2");
    }

    #[test]
    fn merge_enriches_missing_fields() {
        let google = vec![NormalizedBook {
            id: "google:abc".into(),
            source: "google".into(),
            source_id: "abc".into(),
            title: "Test".into(),
            authors: vec![],
            cover_url: None,
            published_year: None,
            page_count: None,
            category: None,
            description: None,
            average_rating: None,
            isbn13: Some("9781234567890".into()),
            isbn10: None,
        }];

        let ol = vec![NormalizedBook {
            id: "openlibrary:/works/OL789W".into(),
            source: "openlibrary".into(),
            source_id: "/works/OL789W".into(),
            title: "Test".into(),
            authors: vec![],
            cover_url: Some("http://ol.com/cover.jpg".into()),
            published_year: None,
            page_count: Some(300),
            category: None,
            description: None,
            average_rating: None,
            isbn13: Some("9781234567890".into()),
            isbn10: None,
        }];

        let merged = merge_results(google, ol);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].cover_url, Some("http://ol.com/cover.jpg".into()));
        assert_eq!(merged[0].page_count, Some(300));
    }

    #[tokio::test]
    async fn search_requires_auth() {
        let app = Router::new().route("/api/books/search", get(unified_search));
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/books/search?query=test")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn detail_requires_auth() {
        let app = Router::new().route("/api/books/external/{source}/{id}", get(external_detail));
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/books/external/google/test-id")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
