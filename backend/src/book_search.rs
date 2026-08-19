use crate::auth::AuthUser;
use crate::db::connect;
use crate::models::Book;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use diesel::PgTextExpressionMethods;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

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

/// Detail-only payload: the lean list `NormalizedBook` plus the richer fields we
/// only fetch when a single book is opened. `#[serde(flatten)]` merges the base
/// fields to the top level, so the frontend sees one flat object.
#[derive(Debug, Clone, Serialize)]
pub struct DetailBook {
    #[serde(flatten)]
    pub base: NormalizedBook,
    pub subtitle: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub categories: Vec<String>,
    pub ratings_count: Option<u32>,
    pub info_link: Option<String>,
    // Only Open Library exposes a resolvable series entity (name + members);
    // Google's `seriesInfo` gives an opaque id with no name, so Google detail
    // leaves this None.
    pub series: Option<SeriesRef>,
}

/// A book's membership in a series: the Open Library series key (e.g. `OL326110L`),
/// its display name, and the book's position within it when known.
#[derive(Debug, Clone, Serialize)]
pub struct SeriesRef {
    pub key: String,
    pub name: String,
    pub position: Option<String>,
}

pub(crate) fn register_routes(router: Router) -> Router {
    router
        .route("/api/books/search", get(unified_search))
        .route("/api/books/trending", get(trending))
        .route("/api/books/external/{source}/{id}", get(external_detail))
        .route("/api/books/editions/{id}", get(work_editions))
        .route("/api/series/{key}", get(series_detail))
        .route("/api/authors/info", get(author_info))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Deserialize)]
pub struct AuthorQuery {
    pub name: String,
}

/// Open Library's public record for an author, keyed by the display name the
/// user's own books carry. Responds with `null` when no record matches, which
/// the author screen renders as "nothing to show" rather than as an error.
///
/// The works in the response are the ones the user does not own yet — that is
/// what makes them worth a row on a screen whose other half is the shelf.
pub(crate) async fn author_info(
    auth: AuthUser,
    Query(params): Query<AuthorQuery>,
) -> impl IntoResponse {
    let client = crate::HTTP.clone();
    let mut info = match crate::open_library_client::find_author(&client, &params.name).await {
        Some(i) => i,
        None => return (StatusCode::OK, Json(Value::Null)),
    };

    let owned = crate::books::books_by_author(auth.0, &params.name);
    info.works.retain(|work| !owns_work(&owned, work));

    (StatusCode::OK, Json(json!(info)))
}

/// Matches an Open Library work against the user's books by the stored catalog
/// id first, then by title.
/// ponytail: a translation carries neither the work key nor the original title,
/// so "Harry Potter und der Feuerkelch" does not hide "…and the Goblet of Fire".
/// Linking the book to its work in the catalog is what fixes that, per book.
fn owns_work(owned: &[Book], work: &NormalizedBook) -> bool {
    let work_key = work.source_id.trim_start_matches('/');
    owned.iter().any(|book| {
        book.open_library_id
            .as_deref()
            .is_some_and(|id| id.trim_start_matches('/') == work_key)
            || book
                .title
                .as_deref()
                .is_some_and(|title| same_title(title, &work.title))
    })
}

/// Compares a shelved title against a catalog title. Either one may carry what
/// the other leaves out — a GoodReads import appends the series ("… Prisoner of
/// Azkaban (Harry Potter, #3)") and the printing ("… Chamber of Secrets:
/// MinaLima Edition") — so a prefix in either direction counts as a match.
/// ponytail: a short catalog title ("Fantastic Beasts") is a prefix of a longer
/// owned one and hides itself. The length floor keeps that to whole titles.
fn same_title(owned: &str, work: &str) -> bool {
    let (owned, work) = (
        crate::open_library_client::squashed(owned),
        crate::open_library_client::squashed(work),
    );
    if owned.len().min(work.len()) < 10 {
        return owned == work;
    }
    owned.starts_with(&work) || work.starts_with(&owned)
}

pub(crate) async fn unified_search(
    auth: AuthUser,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let client = crate::HTTP.clone();
    let query = params.query.clone();

    // The user's own library first, so owned books can win dedup and sort to top.
    let library = library_search(auth.0, &query);
    let lib_keys: Vec<Vec<String>> = library.iter().map(keys_for_library_book).collect();
    let lib_normalized: Vec<NormalizedBook> = library.iter().map(library_to_normalized).collect();

    let google_future = google_search(client.clone(), query.clone());
    let ol_future = crate::open_library_client::search(&client, &query);

    let (google_results, ol_results) = tokio::join!(google_future, ol_future);

    let external = merge_results(google_results, ol_results);
    let merged = merge_with_library(lib_normalized, lib_keys, external);

    (StatusCode::OK, Json(json!(merged)))
}

/// ILIKE-matches the user's own books on title or author. No full-text index —
/// a personal library is small enough for a substring scan.
fn library_search(user_id: Uuid, query: &str) -> Vec<Book> {
    use crate::schema::books::dsl as b;
    let conn = &mut connect();
    let pattern = format!("%{}%", query);
    b::books
        .filter(b::user.eq(user_id))
        .filter(b::title.ilike(&pattern).or(b::author.ilike(&pattern)))
        .limit(25)
        .load::<Book>(conn)
        .unwrap_or_default()
}

pub(crate) fn library_to_normalized(book: &Book) -> NormalizedBook {
    NormalizedBook {
        id: book.id.to_string(),
        source: "library".to_string(),
        source_id: book.id.to_string(),
        title: book.title.clone().unwrap_or_default(),
        authors: book.author.clone().map(|a| vec![a]).unwrap_or_default(),
        cover_url: book.cover_url.clone(),
        published_year: None,
        page_count: book.page_count.map(|p| p as u32),
        category: None,
        description: None,
        average_rating: None,
        isbn13: book.isbn13.clone(),
        isbn10: book.isbn10.clone(),
    }
}

/// Cross-source identity keys for an owned book: ISBNs plus the stored Google /
/// Open Library ids, so an owned book can be matched against an external hit
/// even when they don't share an ISBN.
fn keys_for_library_book(book: &Book) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(v) = &book.isbn13 {
        keys.push(format!("isbn:{v}"));
    }
    if let Some(v) = &book.isbn10 {
        keys.push(format!("isbn:{v}"));
    }
    if let Some(v) = &book.google_books_id {
        keys.push(format!("google:{v}"));
    }
    if let Some(v) = &book.open_library_id {
        keys.push(format!("ol:{v}"));
    }
    keys
}

fn external_keys(book: &NormalizedBook) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(v) = &book.isbn13 {
        keys.push(format!("isbn:{v}"));
    }
    if let Some(v) = &book.isbn10 {
        keys.push(format!("isbn:{v}"));
    }
    match book.source.as_str() {
        "google" => keys.push(format!("google:{}", book.source_id)),
        "openlibrary" => keys.push(format!("ol:{}", book.source_id)),
        _ => {}
    }
    keys
}

/// Prepends the user's library and folds external results into it: an external
/// hit matching an owned book (by any identity key) enriches that owned row's
/// missing fields and is dropped; unmatched hits are appended.
fn merge_with_library(
    library: Vec<NormalizedBook>,
    library_keys: Vec<Vec<String>>,
    external: Vec<NormalizedBook>,
) -> Vec<NormalizedBook> {
    let mut key_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, keys) in library_keys.iter().enumerate() {
        for key in keys {
            key_to_idx.entry(key.clone()).or_insert(i);
        }
    }

    let mut results = library;
    for ext in external {
        match external_keys(&ext)
            .into_iter()
            .find_map(|k| key_to_idx.get(&k).copied())
        {
            Some(idx) => enrich_missing(&mut results[idx], &ext),
            None => results.push(ext),
        }
    }
    results
}

pub(crate) async fn trending(_auth: AuthUser) -> impl IntoResponse {
    let client = crate::HTTP.clone();
    let books = crate::open_library_client::trending(&client, 10).await;
    (StatusCode::OK, Json(json!(books)))
}

pub(crate) async fn external_detail(
    _auth: AuthUser,
    Path((source, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let client = crate::HTTP.clone();

    let result: Option<DetailBook> = match source.as_str() {
        "google" => google_detail(&client, &id).await,
        // `/books/OL…M` is one edition of a work, `/works/OL…W` the work itself.
        // Both are valid stored ids: linking to an edition is how a user pins
        // the language and printing they own.
        "openlibrary" if id.contains("/books/") => {
            crate::open_library_client::get_edition(&client, &id).await
        }
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

/// Lists the editions of an Open Library work, so the user can pick the
/// language and printing they own. Google Books has no equivalent: its volumes
/// are already one per edition.
pub(crate) async fn work_editions(_auth: AuthUser, Path(id): Path<String>) -> impl IntoResponse {
    let client = crate::HTTP.clone();
    let editions = crate::open_library_client::list_editions(&client, &id).await;
    (StatusCode::OK, Json(json!(editions)))
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

async fn google_detail(client: &Client, volume_id: &str) -> Option<DetailBook> {
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
    let base = normalize_google_item(&body)?;
    let vi = &body["volumeInfo"];

    let categories: Vec<String> = vi["categories"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Some(DetailBook {
        base,
        subtitle: vi["subtitle"].as_str().map(String::from),
        publisher: vi["publisher"].as_str().map(String::from),
        published_date: vi["publishedDate"].as_str().map(String::from),
        language: vi["language"].as_str().map(String::from),
        categories,
        ratings_count: vi["ratingsCount"].as_u64().map(|c| c as u32),
        info_link: vi["infoLink"]
            .as_str()
            .or_else(|| vi["canonicalVolumeLink"].as_str())
            .map(String::from),
        series: None,
    })
}

/// Lists the books in an Open Library series, keyed by its series id (e.g.
/// `OL326110L`). Returns the series name and members; members are sorted by
/// publication year — a reasonable default, since Open Library's search facet
/// doesn't return true series order.
pub(crate) async fn series_detail(
    _auth: AuthUser,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let client = crate::HTTP.clone();
    match crate::open_library_client::get_series(&client, &key).await {
        Some((name, books)) => {
            (StatusCode::OK, Json(json!({ "name": name, "books": books }))).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Series not found" })),
        )
            .into_response(),
    }
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

/// Merges the two external sources, **preferring Open Library**: OL rows rank
/// first and win identity on an ISBN collision, so a book present in both
/// catalogs surfaces (and opens) as an Open Library book — required for series,
/// which only OL exposes. A matching Google row backfills fields OL is missing
/// (covers, description, ratings, …); unmatched Google rows are appended after.
fn merge_results(google: Vec<NormalizedBook>, ol: Vec<NormalizedBook>) -> Vec<NormalizedBook> {
    let mut results: Vec<NormalizedBook> = ol;

    let mut ol_isbn_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, book) in results.iter().enumerate() {
        if let Some(key) = isbn_key(book) {
            ol_isbn_to_idx.entry(key).or_insert(i);
        }
    }
    let mut enriched: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for book in google {
        if let Some(key) = isbn_key(&book) {
            if let Some(&idx) = ol_isbn_to_idx.get(&key) {
                if enriched.insert(idx) {
                    enrich_missing(&mut results[idx], &book);
                    continue;
                }
            }
        }
        results.push(book);
    }

    results
}

/// Copies fields from `from` into `target` only where `target` lacks them. Used
/// both to backfill the preferred row from the other external source and to
/// enrich an owned library row from an external hit — hence source-agnostic.
fn enrich_missing(target: &mut NormalizedBook, from: &NormalizedBook) {
    if target.cover_url.is_none() && from.cover_url.is_some() {
        target.cover_url = from.cover_url.clone();
    }
    if target.description.is_none() && from.description.is_some() {
        target.description = from.description.clone();
    }
    if target.page_count.is_none() && from.page_count.is_some() {
        target.page_count = from.page_count;
    }
    if target.category.is_none() && from.category.is_some() {
        target.category = from.category.clone();
    }
    if target.isbn13.is_none() && from.isbn13.is_some() {
        target.isbn13 = from.isbn13.clone();
    }
    if target.isbn10.is_none() && from.isbn10.is_some() {
        target.isbn10 = from.isbn10.clone();
    }
    if target.published_year.is_none() && from.published_year.is_some() {
        target.published_year = from.published_year.clone();
    }
    if target.average_rating.is_none() && from.average_rating.is_some() {
        target.average_rating = from.average_rating;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use tower::ServiceExt;

    #[test]
    fn same_title_matches_across_import_suffixes_and_punctuation() {
        // GoodReads appends the series, Open Library does not.
        assert!(same_title(
            "Harry Potter and the Prisoner of Azkaban (Harry Potter, #3)",
            "Harry Potter and the Prisoner of Azkaban"
        ));
        // The printing is part of the shelved title only.
        assert!(same_title(
            "Harry Potter and the Chamber of Secrets: MinaLima Edition",
            "Harry Potter and the Chamber of Secrets"
        ));
        // The two catalogs disagree on the apostrophe.
        assert!(same_title(
            "Harry Potter and the Philosopher\u{2019}s Stone",
            "Harry Potter and the Philosopher's Stone"
        ));
        assert!(!same_title(
            "Harry Potter und der Feuerkelch",
            "Harry Potter and the Goblet of Fire"
        ));
        // Below the length floor only a full match counts.
        assert!(!same_title("Beasts and more", "Beasts"));
    }

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
        // Open Library wins identity on an ISBN collision...
        assert_eq!(merged[0].source, "openlibrary");
        assert_eq!(merged[0].isbn13, Some("9781234567890".into()));
        // ...and the Google row backfills the fields OL was missing.
        assert_eq!(merged[0].cover_url, Some("http://google.com/cover.jpg".into()));
        assert_eq!(merged[0].description, Some("A test book.".into()));
        assert_eq!(merged[0].average_rating, Some(4.5));
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
        // A single Google row shares its ISBN with two same-source (OL) rows.
        let google = vec![NormalizedBook {
            id: "google:abc".into(),
            source: "google".into(),
            source_id: "abc".into(),
            title: "Book Part 1".into(),
            authors: vec!["Author".into()],
            cover_url: Some("http://google.com/cover.jpg".into()),
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
        ];

        let merged = merge_results(google, ol);
        assert_eq!(merged.len(), 2);
        // Both OL rows survive (preferred source); the Google row backfills only
        // the first, leaving the second untouched.
        assert_eq!(merged[0].source, "openlibrary");
        assert_eq!(merged[0].cover_url, Some("http://google.com/cover.jpg".into()));
        assert_eq!(merged[1].title, "Book Part 2");
        assert_eq!(merged[1].cover_url, None);
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

    fn nb(source: &str, source_id: &str, isbn13: Option<&str>) -> NormalizedBook {
        NormalizedBook {
            id: format!("{source}:{source_id}"),
            source: source.into(),
            source_id: source_id.into(),
            title: "T".into(),
            authors: vec![],
            cover_url: None,
            published_year: None,
            page_count: None,
            category: None,
            description: None,
            average_rating: None,
            isbn13: isbn13.map(String::from),
            isbn10: None,
        }
    }

    #[test]
    fn library_book_wins_and_is_enriched() {
        let mut lib = nb("library", "uuid1", Some("9781234567890"));
        lib.id = "uuid1".into();
        let keys = vec![vec!["isbn:9781234567890".to_string()]];

        let mut ext = nb("google", "vol1", Some("9781234567890"));
        ext.cover_url = Some("http://c/cover.jpg".into());
        ext.published_year = Some("2001".into());

        let merged = merge_with_library(vec![lib], keys, vec![ext]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source, "library");
        assert_eq!(merged[0].id, "uuid1"); // library id preserved for /book/:id routing
        assert_eq!(merged[0].cover_url, Some("http://c/cover.jpg".into()));
        assert_eq!(merged[0].published_year, Some("2001".into()));
    }

    #[test]
    fn library_matches_external_by_google_id_without_isbn() {
        let mut lib = nb("library", "uuid1", None);
        lib.id = "uuid1".into();
        let keys = vec![vec!["google:vol1".to_string()]];

        let merged = merge_with_library(vec![lib], keys, vec![nb("google", "vol1", None)]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source, "library");
    }

    #[test]
    fn distinct_library_and_external_both_kept_library_first() {
        let mut lib = nb("library", "uuid1", Some("1111111111111"));
        lib.id = "uuid1".into();
        let keys = vec![vec!["isbn:1111111111111".to_string()]];

        let merged = merge_with_library(vec![lib], keys, vec![nb("google", "vol1", Some("2222222222222"))]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].source, "library");
        assert_eq!(merged[1].source, "google");
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
