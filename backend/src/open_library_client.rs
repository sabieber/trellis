use crate::book_search::{DetailBook, NormalizedBook, SeriesRef};
use reqwest::Client;
use serde_json::Value;

const OL_BASE: &str = "https://openlibrary.org";

/// Metadata pulled from a work's first edition. All optional — editions are
/// inconsistent about which fields they carry.
#[derive(Default)]
struct EditionMeta {
    isbn13: Option<String>,
    isbn10: Option<String>,
    publisher: Option<String>,
    publish_date: Option<String>,
    page_count: Option<u32>,
    language: Option<String>,
}

pub async fn search(client: &Client, query: &str) -> Vec<NormalizedBook> {
    let url = format!("{}/search.json", OL_BASE);
    let resp = match client
        .get(&url)
        .query(&[("q", query), ("limit", "20")])
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return vec![],
    };

    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    let docs = match body["docs"].as_array() {
        Some(d) => d,
        None => return vec![],
    };

    docs.iter().filter_map(normalize_search_doc).collect()
}

pub async fn trending(client: &Client, limit: u32) -> Vec<NormalizedBook> {
    let url = format!("{}/trending/now.json", OL_BASE);
    let resp = match client
        .get(&url)
        .query(&[("limit", limit.to_string())])
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return vec![],
    };

    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    let works = match body["works"].as_array() {
        Some(w) => w,
        None => return vec![],
    };

    works.iter().filter_map(normalize_search_doc).collect()
}

pub async fn get_work(client: &Client, work_key: &str) -> Option<DetailBook> {
    let key = work_key.trim_start_matches('/');
    let url = format!("{}/{}.json", OL_BASE, key);
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };

    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return None,
    };

    let author_keys: Vec<String> = body["authors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a["author"]["key"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let editions_url = body["editions"]["link"]
        .as_str()
        .map(String::from)
        .unwrap_or_else(|| format!("{}/works/{}/editions.json?limit=1", OL_BASE, key.trim_start_matches("works/")));

    // A work's `series` is an array whose first entry is `{series: {key}, position}`
    // (older records use plain strings, which yield no resolvable key — skipped).
    let series_first = body["series"].as_array().and_then(|a| a.first());
    let series_key = series_first
        .and_then(|s| s["series"]["key"].as_str())
        .map(|k| k.trim_start_matches("/series/").to_string());
    let series_position = series_first.and_then(|s| {
        s["position"]
            .as_str()
            .map(String::from)
            .or_else(|| s["position"].as_i64().map(|n| n.to_string()))
    });

    let (author_names, edition, ratings, series_name) = tokio::join!(
        fetch_author_names(client, &author_keys),
        fetch_edition_meta(client, &editions_url),
        fetch_work_ratings(client, key),
        fetch_series_name(client, series_key.as_deref()),
    );

    let authors = if author_names.is_empty() { None } else { Some(author_names) };

    let series = match (series_key, series_name) {
        (Some(key), Some(name)) => Some(SeriesRef { key, name, position: series_position }),
        _ => None,
    };

    normalize_work(&body, work_key, authors, edition, ratings, series)
}

/// Resolves an Open Library series id (e.g. `OL326110L`) to its display name.
/// None when the key is absent or the lookup fails.
async fn fetch_series_name(client: &Client, key: Option<&str>) -> Option<String> {
    let key = key?;
    let url = format!("{}/series/{}.json", OL_BASE, key);
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    body["name"].as_str().map(String::from)
}

/// Fetches a series' name and its member books (via the `series_key` search
/// facet). Members are sorted by publication year; Open Library's search doesn't
/// return true reading order.
/// ponytail: year sort, upgrade to per-work `position` if members land out of order.
pub async fn get_series(client: &Client, series_key: &str) -> Option<(String, Vec<NormalizedBook>)> {
    let key = series_key.trim_start_matches("/series/").trim_start_matches('/');
    let name = fetch_series_name(client, Some(key)).await?;

    let url = format!("{}/search.json", OL_BASE);
    let resp = client
        .get(&url)
        .query(&[("q", format!("series_key:{}", key).as_str()), ("limit", "50")])
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return Some((name, vec![]));
    }
    let body: Value = resp.json().await.ok()?;
    let mut books: Vec<NormalizedBook> = body["docs"]
        .as_array()
        .map(|docs| docs.iter().filter_map(normalize_search_doc).collect())
        .unwrap_or_default();
    books.sort_by(|a, b| a.published_year.cmp(&b.published_year));

    Some((name, books))
}

/// Fetches the community rating summary for a work (`(average, count)`). This is
/// a separate endpoint from the work JSON, so it costs one extra request — the
/// only way to give Open Library books the star rating Google books already get.
async fn fetch_work_ratings(client: &Client, key: &str) -> (Option<f64>, Option<u32>) {
    let url = format!("{}/{}/ratings.json", OL_BASE, key);
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return (None, None),
    };
    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return (None, None),
    };
    let average = body["summary"]["average"].as_f64();
    let count = body["summary"]["count"].as_u64().map(|c| c as u32);
    (average, count)
}

/// Extracts a 4-digit year from a free-form Open Library date string
/// (e.g. "1999", "September 1999", "1999-09-01").
fn year_from_date(s: &str) -> Option<String> {
    s.as_bytes()
        .windows(4)
        .find(|w| w.iter().all(u8::is_ascii_digit))
        .map(|w| String::from_utf8_lossy(w).into_owned())
}

pub fn is_ol_author_key(s: &str) -> bool {
    let s = s.trim();
    s.len() >= 4
        && s.starts_with("OL")
        && s.ends_with('A')
        && s[2..s.len() - 1].chars().all(|c| c.is_ascii_digit())
}

pub async fn fetch_author_names(client: &Client, keys: &[String]) -> Vec<String> {
    let futures: Vec<_> = keys
        .iter()
        .map(|key| {
            let url = format!("{}/{}.json", OL_BASE, key.trim_start_matches('/'));
            async move {
                match client.get(&url).send().await {
                    Ok(r) if r.status().is_success() => {
                        match r.json::<Value>().await {
                            Ok(b) => b["name"].as_str().map(String::from),
                            Err(_) => None,
                        }
                    }
                    _ => None,
                }
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    results.into_iter().flatten().collect()
}

async fn fetch_edition_meta(client: &Client, editions_url: &str) -> EditionMeta {
    let url = if editions_url.starts_with("http") {
        editions_url.to_string()
    } else {
        format!("{}{}", OL_BASE, editions_url)
    };

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return EditionMeta::default(),
    };

    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return EditionMeta::default(),
    };

    let isbns: Vec<Value> = ed_isbns_as_values(&body).collect();
    let (isbn13, isbn10) = extract_isbns(&isbns);

    let entry = body["entries"].as_array().and_then(|a| a.first());
    let publisher = entry
        .and_then(|e| e["publishers"].as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(String::from);
    let publish_date = entry
        .and_then(|e| e["publish_date"].as_str())
        .map(String::from);
    let page_count = entry
        .and_then(|e| e["number_of_pages"].as_u64())
        .map(|p| p as u32);
    // Language keys look like "/languages/eng"; keep the trailing code.
    let language = entry
        .and_then(|e| e["languages"].as_array())
        .and_then(|a| a.first())
        .and_then(|v| v["key"].as_str())
        .and_then(|k| k.rsplit('/').next())
        .map(String::from);

    EditionMeta {
        isbn13,
        isbn10,
        publisher,
        publish_date,
        page_count,
        language,
    }
}

fn ed_isbns_as_values(body: &Value) -> impl Iterator<Item = Value> + '_ {
    let entry = body["entries"].as_array().and_then(|a| a.first());
    let i13 = entry.and_then(|e| e["isbn_13"].as_array()).into_iter().flatten();
    let i10 = entry.and_then(|e| e["isbn_10"].as_array()).into_iter().flatten();
    i13.chain(i10).cloned()
}

/// Looks up an edition by ISBN and returns its work key together with the page
/// count from the same response. Open Library carries a page count far more often
/// than Google Books for non-English editions, which is why the GoodReads import
/// asks here first. `None` means Open Library has no edition for this ISBN at all.
pub async fn lookup_by_isbn(client: &Client, isbn: &str) -> Option<(String, Option<i32>)> {
    if isbn.is_empty() {
        return None;
    }
    let url = format!("{}/isbn/{}.json", OL_BASE, isbn);
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };
    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return None,
    };

    let pages = body["number_of_pages"]
        .as_i64()
        .filter(|page| *page > 0)
        .map(|page| page as i32);
    let work = body["works"].as_array()?.first()?["key"].as_str()?.to_string();
    Some((work, pages))
}

/// Resolves a cover URL for a book identified by ISBN.
///
/// Strategy:
/// 1. Check if the edition itself has a cover.
/// 2. Follow the edition's work link and check for a cover there.
///
/// This handles the common case where the cover lives on the work
/// page rather than the edition page.
pub async fn resolve_cover_by_isbn(client: &Client, isbn: &str) -> Option<String> {
    if isbn.is_empty() {
        return None;
    }

    // Fetch the edition (ISBN lookup follows redirects).
    let edition_url = format!("{}/isbn/{}.json", OL_BASE, isbn);
    let resp = match client.get(&edition_url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };
    let edition: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return None,
    };

    // 1. Edition-level cover.
    if let Some(url) = first_cover_url(&edition) {
        return Some(url);
    }

    // 2. Follow to work and check there.
    let work_key = edition["works"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v["key"].as_str())?;

    let work_url = format!("{}/{}.json", OL_BASE, work_key.trim_start_matches('/'));
    let wresp = match client.get(&work_url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };
    let work: Value = match wresp.json().await {
        Ok(b) => b,
        Err(_) => return None,
    };

    first_cover_url(&work)
}

/// Formats the ISBN-based Open Library covers URL. Used as a last-resort
/// fallback when no `covers` id could be resolved for an edition or work.
pub fn cover_url_from_isbn(isbn: &str) -> String {
    format!("https://covers.openlibrary.org/b/isbn/{}-M.jpg", isbn)
}

fn first_cover_url(v: &Value) -> Option<String> {
    v["covers"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c.as_i64())
        .map(cover_url_from_id)
}

fn classify_isbn(isbn: &str) -> Option<(& 'static str, String)> {
    let clean: String = isbn.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    match clean.len() {
        13 => Some(("isbn13", clean)),
        10 => Some(("isbn10", clean)),
        _ => None,
    }
}

fn extract_isbns(isbns: &[Value]) -> (Option<String>, Option<String>) {
    let mut isbn13: Option<String> = None;
    let mut isbn10: Option<String> = None;
    for v in isbns {
        if let Some(s) = v.as_str() {
            if let Some((kind, val)) = classify_isbn(s) {
                match kind {
                    "isbn13" if isbn13.is_none() => isbn13 = Some(val),
                    "isbn10" if isbn10.is_none() => isbn10 = Some(val),
                    _ => {}
                }
            }
        }
    }
    (isbn13, isbn10)
}

fn cover_url_from_id(cover_id: i64) -> String {
    format!("https://covers.openlibrary.org/b/id/{}-M.jpg", cover_id)
}

fn normalize_search_doc(doc: &Value) -> Option<NormalizedBook> {
    let title = doc["title"].as_str()?.to_string();
    let key = doc["key"].as_str()?.to_string();
    let source_id = key.clone();

    let authors: Vec<String> = doc["author_name"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .filter(|name| !is_ol_author_key(name))
                .collect()
        })
        .unwrap_or_default();

    let cover_url = doc["cover_i"]
        .as_i64()
        .map(cover_url_from_id);

    let published_year = doc["first_publish_year"]
        .as_i64()
        .map(|y| y.to_string())
        .or_else(|| doc["first_publish_year"].as_str().map(String::from));

    let page_count = doc["number_of_pages_median"].as_u64().map(|p| p as u32);

    let category = doc["subject"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    let isbns: Vec<Value> = doc["isbn"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let (isbn13, isbn10) = extract_isbns(&isbns);

    Some(NormalizedBook {
        id: format!("openlibrary:{}", source_id),
        source: "openlibrary".to_string(),
        source_id,
        title,
        authors,
        cover_url,
        published_year,
        page_count,
        category,
        description: None,
        average_rating: None,
        isbn13,
        isbn10,
    })
}

/// Open Library descriptions are Markdown with reference-style links, a series/
/// source block, and a `----------` divider. Render to HTML with `pulldown-cmark`
/// (handles reference links, nested brackets, and escaping), then sanitize the
/// third-party HTML with `ammonia` so no dangerous tags/attributes or non-web URL
/// schemes (e.g. `javascript:`) reach the frontend's `v-html`. `hr` is dropped so
/// the divider disappears rather than rendering as a rule; links open in a new tab.
fn render_ol_description(raw: &str) -> String {
    let parser = pulldown_cmark::Parser::new(raw);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    let safe = ammonia::Builder::default()
        .rm_tags(std::iter::once("hr"))
        .clean(&html)
        .to_string();

    // ammonia already forces rel="noopener noreferrer"; open external links in a
    // new tab so the SPA isn't navigated away.
    safe.replace("<a ", "<a target=\"_blank\" ").trim().to_string()
}

fn normalize_work(
    body: &Value,
    work_key: &str,
    resolved_authors: Option<Vec<String>>,
    edition: EditionMeta,
    ratings: (Option<f64>, Option<u32>),
    series: Option<SeriesRef>,
) -> Option<DetailBook> {
    let title = body["title"].as_str()?.to_string();
    let source_id = body["key"]
        .as_str()
        .unwrap_or(work_key)
        .to_string();

    let description = body["description"]
        .as_str()
        .map(String::from)
        .or_else(|| {
            body["description"]["value"]
                .as_str()
                .map(String::from)
        })
        .map(|d| render_ol_description(&d))
        .filter(|d| !d.is_empty());

    let subjects: Vec<String> = body["subjects"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let category = subjects.first().cloned();

    let cover_url = body["covers"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_i64())
        .map(cover_url_from_id);

    let authors = resolved_authors.unwrap_or_default();

    // Fixes the list→detail regression where OL detail dropped year and pages:
    // the year comes from the work's first_publish_date, pages from the edition.
    let published_year = body["first_publish_date"]
        .as_str()
        .and_then(year_from_date)
        .or_else(|| edition.publish_date.as_deref().and_then(year_from_date));

    let base = NormalizedBook {
        id: format!("openlibrary:{}", source_id),
        source: "openlibrary".to_string(),
        source_id: source_id.clone(),
        title,
        authors,
        cover_url,
        published_year,
        page_count: edition.page_count,
        category,
        description,
        average_rating: ratings.0,
        isbn13: edition.isbn13,
        isbn10: edition.isbn10,
    };

    Some(DetailBook {
        base,
        subtitle: None,
        publisher: edition.publisher,
        published_date: edition.publish_date,
        language: edition.language,
        categories: subjects,
        ratings_count: ratings.1,
        info_link: Some(format!("{}{}", OL_BASE, source_id)),
        series,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_isbn_lookup_returns_none() {
        assert!(lookup_by_isbn(&Client::new(), "").await.is_none());
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn known_isbn_returns_work_key_and_pages() {
        let (work, pages) = lookup_by_isbn(&Client::new(), "9780140328721")
            .await
            .expect("Fantastic Mr. Fox is on Open Library");
        assert!(work.starts_with("/works/"), "got {}", work);
        assert_eq!(pages, Some(96));
    }

    #[test]
    fn is_ol_author_key_detects_valid_keys() {
        assert!(is_ol_author_key("OL6206333A"));
        assert!(is_ol_author_key("OL123A"));
        assert!(is_ol_author_key("OL1A"));
        assert!(is_ol_author_key(" OL6206333A "));
    }

    #[test]
    fn is_ol_author_key_rejects_non_keys() {
        assert!(!is_ol_author_key("J.K. Rowling"));
        assert!(!is_ol_author_key(""));
        assert!(!is_ol_author_key("OL"));
        assert!(!is_ol_author_key("OLA"));
        assert!(!is_ol_author_key("OLABCA"));
        assert!(!is_ol_author_key("OL123B"));
    }

    #[tokio::test]
    async fn empty_isbn_returns_none_for_cover() {
        let client = Client::new();
        assert!(resolve_cover_by_isbn(&client, "").await.is_none());
    }

    #[test]
    fn render_ol_description_makes_links_clickable_and_drops_plumbing() {
        // Real Open Library shape: prose, source/series block, divider, then
        // reference definitions.
        let raw = "Prose here.\r\n\r\n([source][2])\r\n\r\nPreceded by: [Order of the Phoenix][1]\r\n\
Followed by: [Deathly Hallows][3]\r\n\r\n\r\n----------\r\nContains:\r\n\
[Half-Blood Prince [3/4]](https://openlibrary.org/works/OL27299760W)\r\n\r\n\
  [1]: https://openlibrary.org/works/OL13716955W\r\n  [2]: https://www.jkrowling.com/\r\n\
  [3]: https://openlibrary.org/works/OL82586W";
        let out = render_ol_description(raw);

        // Divider and reference-definition plumbing are gone.
        assert!(!out.contains("----------"));
        assert!(!out.contains("<hr"));
        assert!(!out.contains("[1]:"));
        // No raw Markdown link syntax survives.
        assert!(!out.contains("]("));
        assert!(!out.contains("]["));
        // Series navigation kept, rendered as real anchors opening in a new tab.
        assert!(out.contains("Preceded by:"));
        assert!(out.contains(r#"href="https://openlibrary.org/works/OL13716955W""#));
        assert!(out.contains(">Order of the Phoenix</a>"));
        assert!(out.contains(r#"target="_blank""#));
        assert!(out.contains(r#"rel="noopener noreferrer""#));
        // Inline link with nested brackets in its text is handled.
        assert!(out.contains(r#"href="https://openlibrary.org/works/OL27299760W""#));
        assert!(out.contains("Half-Blood Prince [3/4]"));
    }

    #[test]
    fn render_ol_description_escapes_html() {
        let out = render_ol_description("a < b & c");
        assert!(out.contains("a &lt; b &amp; c"));
        assert!(!out.contains("<b "));
    }

    #[test]
    fn render_ol_description_drops_dangerous_url_schemes() {
        let out = render_ol_description("[click](javascript:alert(1))");
        assert!(!out.contains("javascript"));
        assert!(out.contains("click"));
    }

    #[test]
    fn year_from_date_extracts_year() {
        assert_eq!(year_from_date("1999"), Some("1999".into()));
        assert_eq!(year_from_date("September 1999"), Some("1999".into()));
        assert_eq!(year_from_date("1999-09-01"), Some("1999".into()));
        assert_eq!(year_from_date("n.d."), None);
    }

    #[test]
    fn cover_url_from_id_formats_correct_url() {
        let url = cover_url_from_id(10507461);
        assert_eq!(url, "https://covers.openlibrary.org/b/id/10507461-M.jpg");
    }
}
