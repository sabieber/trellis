//! Cover images: a caching proxy in front of the catalog cover hosts.
//!
//! Open Library answers a cover request with two redirects that end at an
//! archive.org node, which extracts the file from a ZIP archive. Measured time
//! to first byte ranges from 0.5 s to 6 s for the same image, and the final
//! response only carries `max-age=10800` — so every shelf goes slow again
//! three hours later. Hotlinking that from the browser is what makes the app
//! feel slow.
//!
//! This module fetches each cover once, keeps the bytes on disk, and serves
//! them from our own origin with a one-year `immutable` cache.
//!
//! The URL carries a version token derived from the stored `cover_url`, so it
//! changes whenever the cover changes. That matters: `immutable` tells the
//! browser never to revalidate, even on an explicit reload, so it is only
//! correct on a URL that really is immutable. A book that switches its catalog
//! edition (`books::link_catalog`) gets a new `cover_url`, therefore a new
//! token, therefore a new URL — and the browser fetches the new art instead of
//! showing the old one for a year.

use crate::db::connect;
use crate::models::Book;
use crate::schema;
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use diesel::prelude::*;
use std::path::PathBuf;
use uuid::Uuid;

/// The only hosts a stored `cover_url` may point at.
///
/// This list guards both ends. On the way in (`sanitize_cover_url`) it stops a
/// user-supplied `cover_url` from naming an internal address, which would turn
/// the fetch below into a server-side request forgery. On the way out it means
/// the proxy can never be pointed somewhere new after the fact.
const ALLOWED_HOSTS: &[&str] = &[
    "covers.openlibrary.org",
    "books.google.com",
    "books.googleusercontent.com",
];

/// Marks a cover the user uploaded rather than one taken from a catalog.
///
/// The bytes live in the cache directory under the book's id and this token,
/// and there is no upstream to fetch them from. Nothing writes this prefix
/// yet; it exists so an upload slots into the same storage, the same URL shape
/// and the same versioning, and so `books::resolve_cover` — which returns
/// early whenever `cover_url` is set — will not overwrite an uploaded cover
/// with catalog art.
pub const UPLOAD_SCHEME: &str = "upload:";

/// Largest cover we will store. Catalog covers run well under 100 KB; anything
/// past this is not a book cover and should not sit in the cache.
const MAX_COVER_BYTES: usize = 5 * 1024 * 1024;

/// Where the cached bytes live. Override with `COVER_CACHE_DIR`.
///
/// In Docker this must be a volume, or the cache is thrown away on every
/// redeploy. Losing it costs a slow first view per book, not correctness.
pub fn cache_dir() -> PathBuf {
    PathBuf::from(std::env::var("COVER_CACHE_DIR").unwrap_or_else(|_| "cache/covers".to_string()))
}

/// Validates a cover URL and returns it as `https`, or `None` if the host is
/// not one we accept.
///
/// The host is compared whole, so neither a prefix (`covers.openlibrary.org.evil.com`)
/// nor a userinfo segment (`covers.openlibrary.org@evil.com`) nor an added port
/// passes. No other scheme than `http`/`https` is accepted, which rules out
/// `file:` and friends.
///
/// `http` is upgraded rather than rejected. The Google Books API hands out
/// `http://books.google.com/...` thumbnails, and rows imported from it hold
/// that scheme; the same host serves the identical image over `https`, so
/// upgrading keeps those covers working without a migration. This is not a
/// hole in the guard: the host still has to be on the list, so the link-local
/// metadata addresses never pass.
fn upstream_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    ALLOWED_HOSTS
        .contains(&host)
        .then(|| format!("https://{rest}"))
}

/// Filters a `cover_url` on its way into the database.
///
/// Everything that is not an allowed catalog URL or an upload marker becomes
/// `None`. Applied at every write site, this is what lets the fetch below
/// trust the stored value, and it also stops a proxy URL from being written
/// back as if it were an upstream one — a real risk, because the frontend
/// posts a book's `cover_url` back when adding it to a shelf.
pub fn sanitize_cover_url(value: Option<String>) -> Option<String> {
    let value = value.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())?;
    if value.starts_with(UPLOAD_SCHEME) {
        return Some(value);
    }
    upstream_url(&value)
}

/// Derives the version token for a stored `cover_url`.
///
/// FNV-1a, because the token only has to *change* when the URL changes; it is
/// a cache key, not a signature. A hash keeps this free of a new column and a
/// migration — `books` carries no `updated_at` to use instead.
pub fn version_token(cover_url: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in cover_url.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The URL the frontend should render for a book's cover, or `None` when the
/// book has no cover yet. Every handler that sends a book to the frontend puts
/// this in `cover_url` instead of the upstream URL.
pub fn proxy_url(book_id: Uuid, cover_url: Option<&str>) -> Option<String> {
    let cover_url = cover_url?;
    Some(format!(
        "/api/books/{book_id}/cover?v={}",
        version_token(cover_url)
    ))
}

pub(crate) fn register_routes(router: Router) -> Router {
    router.route("/api/books/{id}/cover", get(serve_cover))
}

/// Serves a book's cover, fetching and caching it on the first request.
///
/// This route carries no `AuthUser`. An `<img>` tag cannot send an
/// `Authorization` header, so a bearer-token check is not available here. The
/// book id is a v4 UUID and appears nowhere public, and the response body is
/// catalog cover art — so the id is the capability. Note what that means: the
/// route deliberately returns nothing but image bytes. No title, no author, no
/// error text that would distinguish "no such book" from "book has no cover".
///
/// The `v` query parameter is never read. It exists only to give the browser a
/// new URL when the cover changes; the server always serves the current cover.
async fn serve_cover(Path(id): Path<String>) -> Response {
    let Ok(book_id) = Uuid::parse_str(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let connection = &mut connect();
    let Ok(book) = schema::books::dsl::books
        .filter(schema::books::dsl::id.eq(book_id))
        .first::<Book>(connection)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(cover_url) = book.cover_url.as_deref() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let path = cache_dir().join(format!("{book_id}-{}", version_token(cover_url)));

    if let Ok(bytes) = tokio::fs::read(&path).await {
        return image_response(bytes);
    }

    // An uploaded cover has no upstream. If its bytes are not in the cache
    // they are gone, and re-fetching is not a thing we can do. Rows written
    // before `sanitize_cover_url` existed are validated here too, so the
    // stored value is never trusted on its own.
    let Some(upstream) = upstream_url(cover_url) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(bytes) = fetch_upstream(&upstream).await else {
        // A miss leaves the frontend's `@error` path to run, which draws the
        // typographic cover and asks the backend to resolve a better URL.
        return StatusCode::NOT_FOUND.into_response();
    };

    store(&path, book_id, &bytes).await;
    image_response(bytes)
}

/// Downloads a cover, rejecting anything that is not a plausible image.
async fn fetch_upstream(url: &str) -> Option<Vec<u8>> {
    let response = crate::HTTP.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    if response.content_length().is_some_and(|n| n as usize > MAX_COVER_BYTES) {
        return None;
    }

    let bytes = response.bytes().await.ok()?;
    // Checked again after reading: `content-length` is a claim, not a promise.
    if bytes.len() > MAX_COVER_BYTES {
        return None;
    }
    // Open Library answers a missing cover with a 1×1 GIF, which a size floor
    // catches without decoding anything.
    if bytes.len() < 256 || sniff_content_type(&bytes).is_none() {
        return None;
    }
    if is_google_placeholder(&bytes) {
        return None;
    }
    Some(bytes.to_vec())
}

/// Writes the bytes into the cache and drops the book's older covers.
///
/// The write goes to a temporary file and is renamed into place, so a reader
/// never sees a half-written cover. Two requests for the same missing cover
/// both fetch and both write; the rename makes that harmless.
///
/// A failure here is not reported. The cover was already fetched, so the
/// request can be answered — the only cost is that the next one refetches.
async fn store(path: &std::path::Path, book_id: Uuid, bytes: &[u8]) {
    let dir = cache_dir();
    if tokio::fs::create_dir_all(&dir).await.is_err() {
        return;
    }

    let temporary = dir.join(format!("{book_id}.{}.tmp", std::process::id()));
    if tokio::fs::write(&temporary, bytes).await.is_err() {
        return;
    }
    if tokio::fs::rename(&temporary, path).await.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
        return;
    }

    // The token changed, so the previous cover of this book is now unreachable.
    // Without this every edition switch would leave a file behind forever.
    let prefix = format!("{book_id}-");
    let keep = path.file_name().unwrap_or_default();
    if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name();
            if name != keep && name.to_string_lossy().starts_with(&prefix) {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
    }
}

/// True for Google Books' "image not available" graphic.
///
/// Google answers a content request for a volume it has no cover for with a
/// 128×170 grayscale PNG rather than a 404, so the size floor above does not
/// catch it — it is a valid, ~1.2 KB image. Caching it would paint that
/// placeholder onto the shelf instead of the typographic cover.
///
/// The dimensions come out of the PNG header (IHDR width and height are the
/// four-byte big-endian fields at offsets 16 and 20), which is stable in a way
/// the byte length is not. `useCoverImage` applies the same 128×170 rule in the
/// browser for the covers that are still hotlinked; this is that rule moved to
/// where the bytes are cached.
fn is_google_placeholder(bytes: &[u8]) -> bool {
    if sniff_content_type(bytes) != Some("image/png") || bytes.len() < 24 {
        return false;
    }
    let dimension = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    dimension(16) == 128 && dimension(20) == 170
}

/// Identifies the image format from its magic bytes.
///
/// The cache stores only the image, so the type is recovered on the way out
/// rather than kept in a sidecar file. Returning `None` is also how a fetched
/// body is rejected: whatever it is, it is not an image we should serve.
fn sniff_content_type(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg"),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("image/png"),
        [b'G', b'I', b'F', b'8', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        _ => None,
    }
}

fn image_response(bytes: Vec<u8>) -> Response {
    let content_type = sniff_content_type(&bytes).unwrap_or("application/octet-stream");
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            // Safe only because the token in the URL changes with the cover.
            (
                header::CACHE_CONTROL,
                "public, max-age=31536000, immutable",
            ),
        ],
        bytes,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_catalog_cover_hosts() {
        let url = "https://covers.openlibrary.org/b/id/10507461-M.jpg";
        assert_eq!(upstream_url(url).as_deref(), Some(url));
        let url = "https://books.google.com/books/content?id=abc&img=1";
        assert_eq!(upstream_url(url).as_deref(), Some(url));
    }

    /// The Google Books API hands out `http` thumbnails, so rows imported from
    /// it carry that scheme. Rejecting them would drop working cover art.
    #[test]
    fn upgrades_http_on_an_allowed_host() {
        assert_eq!(
            upstream_url("http://books.google.com/books/content?id=abc&img=1").as_deref(),
            Some("https://books.google.com/books/content?id=abc&img=1")
        );
    }

    #[test]
    fn rejects_hosts_that_only_look_right() {
        // The upgrade above must not become a way in for anything else.
        assert_eq!(upstream_url("http://169.254.169.254/latest/meta-data/"), None);
        assert_eq!(upstream_url("http://localhost:5174/api/users"), None);
        // A suffix, a userinfo segment and an added port are all different hosts.
        assert_eq!(upstream_url("https://covers.openlibrary.org.evil.com/x.jpg"), None);
        assert_eq!(upstream_url("https://covers.openlibrary.org@evil.com/x.jpg"), None);
        assert_eq!(upstream_url("https://covers.openlibrary.org:8080/x.jpg"), None);
        assert_eq!(upstream_url("file:///etc/passwd"), None);
        assert_eq!(upstream_url("/api/books/x/cover"), None);
    }

    #[test]
    fn sanitize_keeps_catalog_urls_and_upload_markers() {
        let url = "https://covers.openlibrary.org/b/id/1-M.jpg".to_string();
        assert_eq!(sanitize_cover_url(Some(url.clone())), Some(url));

        let upload = "upload:0f9a".to_string();
        assert_eq!(sanitize_cover_url(Some(upload.clone())), Some(upload));
    }

    #[test]
    fn sanitize_drops_blanks_and_foreign_urls() {
        assert_eq!(sanitize_cover_url(None), None);
        assert_eq!(sanitize_cover_url(Some("   ".to_string())), None);
        assert_eq!(sanitize_cover_url(Some("https://evil.com/x.jpg".to_string())), None);
    }

    /// The frontend posts a book's `cover_url` back when adding it to a shelf.
    /// Without this the proxy URL would be stored as if it were an upstream one,
    /// and the next fetch would try to resolve it against a catalog host.
    #[test]
    fn sanitize_drops_a_proxy_url_posted_back() {
        let proxied = proxy_url(Uuid::new_v4(), Some("https://covers.openlibrary.org/b/id/1-M.jpg"));
        assert_eq!(sanitize_cover_url(proxied), None);
    }

    #[test]
    fn token_changes_when_the_cover_changes() {
        let first = version_token("https://covers.openlibrary.org/b/id/1-M.jpg");
        let second = version_token("https://covers.openlibrary.org/b/id/2-M.jpg");
        assert_ne!(first, second, "an edition switch must produce a new URL");
        assert_eq!(first.len(), 16);
    }

    #[test]
    fn token_is_stable_for_the_same_cover() {
        let url = "https://covers.openlibrary.org/b/id/1-M.jpg";
        assert_eq!(version_token(url), version_token(url));
    }

    #[test]
    fn proxy_url_carries_the_book_id_and_the_token() {
        let id = Uuid::new_v4();
        let url = "https://covers.openlibrary.org/b/id/1-M.jpg";
        assert_eq!(
            proxy_url(id, Some(url)),
            Some(format!("/api/books/{id}/cover?v={}", version_token(url)))
        );
    }

    #[test]
    fn proxy_url_is_none_without_a_cover() {
        assert_eq!(proxy_url(Uuid::new_v4(), None), None);
    }

    #[test]
    fn sniffs_the_formats_the_catalogs_serve() {
        assert_eq!(sniff_content_type(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(
            sniff_content_type(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            Some("image/png")
        );
        assert_eq!(sniff_content_type(b"GIF89a..."), Some("image/gif"));
        assert_eq!(sniff_content_type(b"RIFF____WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff_content_type(b"<!doctype html>"), None);
    }

    /// Builds a PNG header with the given dimensions — enough of one for
    /// `is_google_placeholder`, which only reads the IHDR size fields.
    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    /// Google answers a volume it has no cover for with a 128×170 grayscale
    /// PNG and a `200`, so this is the only thing separating "no cover" from a
    /// real one.
    #[test]
    fn rejects_the_google_not_available_placeholder() {
        assert!(is_google_placeholder(&png_header(128, 170)));
    }

    #[test]
    fn keeps_real_covers_and_other_formats() {
        assert!(!is_google_placeholder(&png_header(128, 171)));
        assert!(!is_google_placeholder(&png_header(400, 600)));
        // A JPEG is never the placeholder, whatever its size fields decode to.
        assert!(!is_google_placeholder(&[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(!is_google_placeholder(b"too short"));
    }
}
