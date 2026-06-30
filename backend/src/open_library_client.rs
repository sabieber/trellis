use crate::book_search::NormalizedBook;
use reqwest::Client;
use serde_json::Value;

const OL_BASE: &str = "https://openlibrary.org";

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

pub async fn get_work(client: &Client, work_key: &str) -> Option<NormalizedBook> {
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

    let (author_names, isbn_pair) = tokio::join!(
        fetch_author_names(client, &author_keys),
        fetch_edition_isbns(client, &editions_url),
    );

    let authors = if author_names.is_empty() { None } else { Some(author_names) };

    normalize_work(&body, work_key, authors, isbn_pair)
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

async fn fetch_edition_isbns(client: &Client, editions_url: &str) -> (Option<String>, Option<String>) {
    let url = if editions_url.starts_with("http") {
        editions_url.to_string()
    } else {
        format!("{}{}", OL_BASE, editions_url)
    };

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return (None, None),
    };

    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return (None, None),
    };

    body["entries"]
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|ed| ed["isbn_13"].as_array().or_else(|| ed["isbn_10"].as_array()))
        .map(|_| {
            let isbns: Vec<Value> = ed_isbns_as_values(&body).collect();
            extract_isbns(&isbns)
        })
        .unwrap_or((None, None))
}

fn ed_isbns_as_values(body: &Value) -> impl Iterator<Item = Value> + '_ {
    let entry = body["entries"].as_array().and_then(|a| a.first());
    let i13 = entry.and_then(|e| e["isbn_13"].as_array()).into_iter().flatten();
    let i10 = entry.and_then(|e| e["isbn_10"].as_array()).into_iter().flatten();
    i13.chain(i10).cloned()
}

#[allow(dead_code)]
pub async fn lookup_id_by_isbn(client: &Client, isbn: &str) -> Option<String> {
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

    let works = body["works"].as_array()?;
    let first = works.first()?;
    first["key"].as_str().map(|k| k.to_string())
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

fn normalize_work(
    body: &Value,
    work_key: &str,
    resolved_authors: Option<Vec<String>>,
    edition_isbns: (Option<String>, Option<String>),
) -> Option<NormalizedBook> {
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
        });

    let subjects: Vec<String> = body["subjects"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let category = subjects.into_iter().next();

    let cover_url = body["covers"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_i64())
        .map(cover_url_from_id);

    let authors = resolved_authors.unwrap_or_default();

    Some(NormalizedBook {
        id: format!("openlibrary:{}", source_id),
        source: "openlibrary".to_string(),
        source_id,
        title,
        authors,
        cover_url,
        published_year: None,
        page_count: None,
        category,
        description,
        average_rating: None,
        isbn13: edition_isbns.0,
        isbn10: edition_isbns.1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
