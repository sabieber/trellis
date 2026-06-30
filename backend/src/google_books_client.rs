use reqwest::Client;
use serde_json::Value;

pub async fn lookup_id_by_isbn(client: &Client, isbn: &str) -> Option<String> {
    if isbn.is_empty() {
        return None;
    }
    let max_retries = 3;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_millis(500 * (1 << (attempt - 1)));
            tokio::time::sleep(delay).await;
        }
        let mut req = client
            .get("https://www.googleapis.com/books/v1/volumes")
            .query(&[
                ("q", &format!("isbn:{}", isbn)),
                ("maxResults", &"1".to_string()),
            ]);
        if let Ok(key) = std::env::var("GOOGLE_BOOKS_API_KEY") {
            req = req.query(&[("key", key)]);
        }
        let resp = match req.send().await {
            Ok(r) => r,
            Err(_) if attempt < max_retries => continue,
            Err(_) => return None,
        };
        if resp.status().as_u16() == 429 && attempt < max_retries {
            continue;
        }
        if !resp.status().is_success() {
            return None;
        }
        let json: Value = match resp.json().await {
            Ok(j) => j,
            Err(_) if attempt < max_retries => continue,
            Err(_) => return None,
        };
        return json["items"][0]["id"].as_str().map(|s| s.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
