use crate::book_search::{DetailBook, NormalizedBook, SeriesRef};
use reqwest::Client;
use serde::Serialize;
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

    let entry = body["entries"].as_array().and_then(|a| a.first());
    let (isbn13, isbn10) = entry.map(entry_isbns).unwrap_or((None, None));
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

fn entry_isbns(entry: &Value) -> (Option<String>, Option<String>) {
    let i13 = entry["isbn_13"].as_array().into_iter().flatten();
    let i10 = entry["isbn_10"].as_array().into_iter().flatten();
    let isbns: Vec<Value> = i13.chain(i10).cloned().collect();
    extract_isbns(&isbns)
}

/// Maps one edition document — a member of `editions.json` or an edition fetched
/// on its own; they carry the same fields — to a `DetailBook`. Work-level data
/// (description, subjects, rating, series, authors) is not part of an edition;
/// `get_edition` merges it in.
fn normalize_edition_entry(entry: &Value) -> Option<DetailBook> {
    let source_id = entry["key"].as_str()?.to_string();
    let title = entry["title"].as_str()?.to_string();
    let (isbn13, isbn10) = entry_isbns(entry);
    let publish_date = entry["publish_date"].as_str().map(String::from);

    Some(DetailBook {
        base: NormalizedBook {
            id: format!("openlibrary:{}", source_id),
            source: "openlibrary".to_string(),
            source_id,
            title,
            authors: vec![],
            cover_url: entry["covers"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_i64())
                .map(cover_url_from_id),
            published_year: publish_date.as_deref().and_then(year_from_date),
            page_count: entry["number_of_pages"].as_u64().map(|p| p as u32),
            category: None,
            description: None,
            average_rating: None,
            isbn13,
            isbn10,
        },
        subtitle: entry["subtitle"].as_str().map(String::from),
        publisher: entry["publishers"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .map(String::from),
        published_date: publish_date,
        language: entry["languages"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v["key"].as_str())
            .and_then(|k| k.rsplit('/').next())
            .map(String::from),
        categories: vec![],
        ratings_count: None,
        info_link: None,
        series: None,
    })
}

/// Lists the editions of a work.
///
/// Open Library models translations and reprints as separate editions of one
/// work, and the work itself carries no language. Picking "the German one" is
/// therefore an edition choice only the user can make.
pub async fn list_editions(client: &Client, work_key: &str) -> Vec<DetailBook> {
    let key = work_key.trim_start_matches('/');
    let url = format!("{}/{}/editions.json?limit=300", OL_BASE, key);
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return vec![],
    };
    let body: Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    body["entries"]
        .as_array()
        .map(|entries| entries.iter().filter_map(normalize_edition_entry).collect())
        .unwrap_or_default()
}

/// Fetches one edition (`/books/OL…M`) and merges its work's shared data on top.
/// The edition wins for everything it states — title, cover, pages, ISBN,
/// publisher, language — which is the whole point of picking one.
pub async fn get_edition(client: &Client, edition_key: &str) -> Option<DetailBook> {
    let key = edition_key.trim_start_matches('/');
    let url = format!("{}/{}.json", OL_BASE, key);
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };
    let body: Value = resp.json().await.ok()?;

    let mut detail = normalize_edition_entry(&body)?;

    let work_key = body["works"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|w| w["key"].as_str());
    if let Some(work) = match work_key {
        Some(k) => get_work(client, k).await,
        None => None,
    } {
        detail.base.authors = work.base.authors;
        detail.base.description = work.base.description;
        detail.base.category = work.base.category;
        detail.base.average_rating = work.base.average_rating;
        detail.ratings_count = work.ratings_count;
        detail.categories = work.categories;
        detail.series = work.series;
        // Fields a sparse edition leaves empty still beat showing nothing.
        detail.base.cover_url = detail.base.cover_url.or(work.base.cover_url);
        detail.base.page_count = detail.base.page_count.or(work.base.page_count);
        detail.base.published_year = detail.base.published_year.or(work.base.published_year);
    }

    Some(detail)
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

/// An outbound link for an author: the author's own pages from the record's
/// `links`, plus the reader-facing sites from its `remote_ids`.
#[derive(Debug, Serialize)]
pub struct AuthorLink {
    pub title: String,
    pub url: String,
}

/// The reader-facing sites we build a link for out of an author's `remote_ids`.
/// The rest of that map is library-science plumbing (viaf, isni, gnd, …) that no
/// reader follows.
const REMOTE_ID_SITES: [(&str, &str, &str); 5] = [
    ("goodreads", "Goodreads", "https://www.goodreads.com/author/show/"),
    ("librarything", "LibraryThing", "https://www.librarything.com/author/"),
    ("storygraph", "The StoryGraph", "https://app.thestorygraph.com/authors/"),
    ("amazon", "Amazon", "https://www.amazon.com/-/e/"),
    ("wikidata", "Wikidata", "https://www.wikidata.org/wiki/"),
];

/// The public Open Library record for an author. Every field beside the key and
/// the name is optional — author records are sparse.
#[derive(Debug, Serialize)]
pub struct AuthorInfo {
    pub key: String,
    pub name: String,
    /// Rendered and sanitized HTML, same treatment as a work's description.
    pub bio: Option<String>,
    pub birth_date: Option<String>,
    pub death_date: Option<String>,
    pub photo_url: Option<String>,
    /// Pen names and spelling variants, minus the ones that only re-order or
    /// re-case the name itself.
    pub alternate_names: Vec<String>,
    pub links: Vec<AuthorLink>,
    /// Works Open Library knows, not works the user owns.
    pub work_count: Option<u64>,
    /// The author's most-read works, in Open Library's reading-log order. The
    /// caller drops the ones the user owns.
    pub works: Vec<NormalizedBook>,
    /// The series those works belong to. Open Library files series on the work,
    /// so this is what the works above are part of, not a complete bibliography.
    pub series: Vec<AuthorSeries>,
}

/// A series an author writes in.
#[derive(Debug, Serialize)]
pub struct AuthorSeries {
    pub name: String,
    /// Set when Open Library files the series as an entity of its own, which is
    /// what makes it linkable. The `series:` subjects carry a name only.
    pub key: Option<String>,
}

/// One page of an author's works: what the search returned, how many there are
/// in total, and the two places the works name their series.
struct WorksPage {
    works: Vec<NormalizedBook>,
    total: Option<u64>,
    /// Keys of series filed as entities (`/series/OL…L`).
    series_keys: Vec<String>,
    /// Series named by a `series:<name>` subject. Sanderson's works carry these
    /// and no entity at all, so both sources are needed.
    series_subjects: Vec<String>,
}

/// How many of the author's works the works search asks for. The caller drops
/// the owned ones from this list, so it must hold more than a screen's worth.
const AUTHOR_WORKS_LIMIT: &str = "16";

/// Resolves an author by display name: one work search for the author key, then
/// the author record and a works search on that key.
///
/// The author record must carry the requested name (see `same_name`), otherwise
/// the search's best guess is dropped. A wrong biography is worse than none.
///
/// This goes through the work search, not `/search/authors.json`: the author
/// search answered the same query in 4s and in 83s on two tries, the work search
/// in 1s.
/// ponytail: no cache, three requests per page view. Add one if Open Library
/// throttles us.
pub async fn find_author(client: &Client, name: &str) -> Option<AuthorInfo> {
    let key = resolve_author_key(client, name).await?;

    // The works search doubles as the work count. The count must come from a
    // search on the key, not from the name search above: a name query only
    // counts the works spelling the name the same way (27 of J. K. Rowling's
    // 418).
    let (record, works_page) =
        tokio::join!(fetch_author_record(client, &key), fetch_works(client, &key));
    let record = record?;
    let record_name = record["name"].as_str()?;
    let alternates = record["alternate_names"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    if !same_name(name, record_name) && !alternates.iter().any(|alt| same_name(name, alt)) {
        return None;
    }

    let keyed = fetch_series_refs(client, &works_page.series_keys).await;
    let series = merge_series(keyed, works_page.series_subjects);

    Some(AuthorInfo {
        name: record_name.to_string(),
        bio: record["bio"]
            .as_str()
            .or_else(|| record["bio"]["value"].as_str())
            .map(render_ol_description),
        birth_date: record["birth_date"].as_str().map(String::from),
        death_date: record["death_date"].as_str().map(String::from),
        // A deleted photo is stored as a negative id, so filter before formatting.
        photo_url: record["photos"]
            .as_array()
            .and_then(|a| a.iter().filter_map(Value::as_i64).find(|id| *id > 0))
            .map(|id| format!("https://covers.openlibrary.org/a/id/{}-M.jpg", id)),
        alternate_names: pick_alternate_names(record_name, &alternates),
        links: collect_links(&record),
        work_count: works_page.total,
        works: works_page.works,
        series,
        key,
    })
}

/// Resolves series keys to their names, dropping the ones Open Library cannot
/// name. The lookups run together — an author has a handful of series at most.
async fn fetch_series_refs(client: &Client, keys: &[String]) -> Vec<AuthorSeries> {
    let lookups = keys.iter().map(|key| async move {
        Some(AuthorSeries {
            name: fetch_series_name(client, Some(key)).await?,
            key: Some(key.clone()),
        })
    });
    futures::future::join_all(lookups).await.into_iter().flatten().collect()
}

/// How many series an author's panel shows. A handful names the shelf; a list of
/// every printing variant does not.
const AUTHOR_SERIES_LIMIT: usize = 8;

/// Appends the series known only by name to the ones known as entities, and
/// drops the repeats. The two sources spell the same series differently —
/// `Harry_Potter` as a subject against "Harry Potter" as an entity — so they
/// compare squashed.
fn merge_series(keyed: Vec<AuthorSeries>, subject_names: Vec<String>) -> Vec<AuthorSeries> {
    let mut series = keyed;
    for name in subject_names {
        if series.iter().any(|s| squashed(&s.name) == squashed(&name)) {
            continue;
        }
        series.push(AuthorSeries { name, key: None });
    }
    series.truncate(AUTHOR_SERIES_LIMIT);
    series
}

/// Keeps the alternate names that say something new: a pen name or a foreign
/// spelling. Drops the ones that only re-order or re-case the words of the name
/// itself ("King, Stephen"), the duplicates among themselves, and the long
/// composite entries that Open Library's importers leave behind.
/// ponytail: length cap as the junk filter. Nothing shorter distinguishes
/// "King, Stephen (1947- )" from a real variant.
fn pick_alternate_names(name: &str, alternates: &[&str]) -> Vec<String> {
    let name_words = word_set(name);
    let mut seen = vec![name_words.clone()];
    // A name sharing no word with the record's own is a pen name ("Robert
    // Galbraith"), which is the interesting kind. Those go first, so the limit
    // below cuts the spelling variants instead of them.
    let (mut pen_names, mut variants) = (Vec::new(), Vec::new());
    for alt in alternates {
        let words = word_set(alt);
        if alt.chars().count() > 40 || words.is_empty() || seen.contains(&words) {
            continue;
        }
        if words.iter().any(|w| name_words.contains(w)) {
            variants.push(alt.to_string());
        } else {
            pen_names.push(alt.to_string());
        }
        seen.push(words);
    }
    pen_names.append(&mut variants);
    pen_names.truncate(5);
    pen_names
}

/// The words of a name, lowercased, stripped of punctuation and sorted, so two
/// orderings of the same name compare equal.
fn word_set(s: &str) -> Vec<String> {
    let mut words: Vec<String> = s
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Collects the author's outbound links: the record's own `links` first (the
/// official site, encyclopedia entries), then the reader-facing `remote_ids`.
fn collect_links(record: &Value) -> Vec<AuthorLink> {
    let mut links: Vec<AuthorLink> = record["links"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| {
                    let url = l["url"].as_str()?;
                    // Only web links reach the frontend's href.
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return None;
                    }
                    // Editors park housekeeping links on the record
                    // ("needs-merge-openlibrary"), and the author's own Open
                    // Library page is a link the caller adds itself.
                    if url.contains("openlibrary.org") {
                        return None;
                    }
                    Some(AuthorLink {
                        title: l["title"].as_str().unwrap_or("Link").to_string(),
                        url: url.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    for (field, title, prefix) in REMOTE_ID_SITES {
        if let Some(id) = record["remote_ids"][field].as_str() {
            links.push(AuthorLink {
                title: title.to_string(),
                url: format!("{}{}", prefix, id),
            });
        }
    }

    links
}

/// Finds the Open Library key of the author with this name.
///
/// The work search answers first because it is the quick one (~1s, against 4s
/// and 83s measured on two tries of the author search). It matches the author
/// name literally though, so a work by "J.R.R. Tolkien" is no hit for "J. R. R.
/// Tolkien" — and the top hit for that spelling is a conference named after him.
/// When no name in its results is the one we asked for, the author search
/// decides: that endpoint knows the spellings, and only the odd names pay its
/// latency.
async fn resolve_author_key(client: &Client, name: &str) -> Option<String> {
    let url = format!("{}/search.json", OL_BASE);
    let body = get_json(
        client,
        &url,
        &[
            ("q", format!("author:\"{}\"", name.replace('"', "")).as_str()),
            ("limit", "5"),
            ("fields", "author_key,author_name"),
        ],
    )
    .await;

    // A work lists every one of its authors, and `author_key` and `author_name`
    // are parallel arrays — so the key we want sits at the index of the matching
    // name, which is not always the first.
    if let Some(body) = body {
        let hit = body["docs"].as_array().into_iter().flatten().find_map(|doc| {
            let names = doc["author_name"].as_array()?;
            let index = names
                .iter()
                .position(|n| n.as_str().is_some_and(|n| same_name(name, n)))?;
            doc["author_key"].as_array()?.get(index)?.as_str().map(String::from)
        });
        if hit.is_some() {
            return hit;
        }
    }

    let url = format!("{}/search/authors.json", OL_BASE);
    let body = get_json(client, &url, &[("q", name), ("limit", "5")]).await?;
    body["docs"].as_array()?.iter().find_map(|doc| {
        if same_name(name, doc["name"].as_str()?) {
            doc["key"].as_str().map(|k| k.trim_start_matches("/authors/").to_string())
        } else {
            None
        }
    })
}

/// GETs a JSON body, or `None` on any failure. Every Open Library call is
/// optional enrichment, so a failure is an absent panel, never an error.
async fn get_json(client: &Client, url: &str, query: &[(&str, &str)]) -> Option<Value> {
    let resp = client
        .get(url)
        .query(query)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().await.ok()
}

/// Compares two author names without case, punctuation and spacing, so
/// "J.K. Rowling" matches "J. K. Rowling".
fn same_name(a: &str, b: &str) -> bool {
    squashed(a) == squashed(b)
}

/// A title or name stripped to its lowercase alphanumerics, for comparing two
/// spellings of the same thing. Two catalogs differ on apostrophes, hyphens and
/// spacing ("Philosopher's" vs "Philosopher’s") far more often than on letters.
pub fn squashed(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Searches the works filed under an author key, most-read first, and returns
/// them together with the total the search reports.
async fn fetch_works(client: &Client, key: &str) -> WorksPage {
    let url = format!("{}/search.json", OL_BASE);
    let body = match get_json(
        client,
        &url,
        &[
            ("q", format!("author_key:{}", key).as_str()),
            ("limit", AUTHOR_WORKS_LIMIT),
            ("sort", "readinglog"),
            (
                "fields",
                "key,title,cover_i,first_publish_year,author_name,number_of_pages_median,series_key,subject",
            ),
        ],
    )
    .await
    {
        Some(b) => b,
        None => {
            return WorksPage {
                works: vec![],
                total: None,
                series_keys: vec![],
                series_subjects: vec![],
            }
        }
    };

    let docs = body["docs"].as_array().cloned().unwrap_or_default();

    let mut series_keys: Vec<String> = docs
        .iter()
        .filter_map(|doc| doc["series_key"].as_array())
        .flatten()
        .filter_map(|k| k.as_str().map(String::from))
        .collect();
    series_keys.sort();
    series_keys.dedup();

    let series_subjects: Vec<String> = docs
        .iter()
        .filter_map(|doc| doc["subject"].as_array())
        .flatten()
        .filter_map(|s| series_subject_name(s.as_str()?))
        .collect();

    WorksPage {
        works: docs.iter().filter_map(normalize_search_doc).collect(),
        total: body["numFound"].as_u64(),
        series_keys,
        series_subjects,
    }
}

/// Reads the series name out of a `series:<name>` subject. Open Library writes
/// these with the spaces replaced by underscores about as often as not.
fn series_subject_name(subject: &str) -> Option<String> {
    let rest = subject.strip_prefix("series:").or_else(|| subject.strip_prefix("Series:"))?;
    let name = rest.replace('_', " ").trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Fetches the raw author record. `bio` is a plain string on older records and a
/// `{type, value}` object on newer ones, so the caller reads both shapes.
async fn fetch_author_record(client: &Client, key: &str) -> Option<Value> {
    let url = format!("{}/authors/{}.json", OL_BASE, key);
    get_json(client, &url, &[]).await
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

    #[test]
    fn pick_alternate_names_drops_reorderings_and_junk() {
        let alternates = vec![
            "Stephen king",             // only case
            "King, Stephen",            // only order
            "Richard Bachman",          // pen name — keep
            "Stiven King",              // foreign spelling — keep
            "Richard Bachman ( LI CHA BA HE MAN ) . Stephen King ( SI DI FEN", // junk
        ];
        assert_eq!(
            pick_alternate_names("Stephen King", &alternates),
            vec!["Richard Bachman", "Stiven King"]
        );
    }

    #[test]
    fn merge_series_keeps_the_linkable_spelling_of_a_repeat() {
        let keyed = vec![AuthorSeries {
            name: "Harry Potter".into(),
            key: Some("OL326110L".into()),
        }];
        let subjects = vec![
            series_subject_name("series:Harry_Potter").unwrap(),
            series_subject_name("series:The Mistborn Saga").unwrap(),
            series_subject_name("series:The Mistborn Saga").unwrap(),
        ];
        let merged = merge_series(keyed, subjects);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].key.as_deref(), Some("OL326110L"));
        assert_eq!(merged[1].name, "The Mistborn Saga");
        assert!(merged[1].key.is_none());
    }

    #[test]
    fn collect_links_takes_record_links_and_known_remote_ids() {
        let record = serde_json::json!({
            "links": [
                {"title": "Official Site", "url": "https://jkrowling.com/"},
                {"title": "Broken", "url": "javascript:alert(1)"},
            ],
            "remote_ids": {"goodreads": "1077326", "viaf": "116796842"},
        });
        let links = collect_links(&record);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://jkrowling.com/");
        assert_eq!(links[1].url, "https://www.goodreads.com/author/show/1077326");
    }

    #[test]
    fn same_name_ignores_case_punctuation_and_spacing() {
        assert!(same_name("J.K. Rowling", "J. K. Rowling"));
        assert!(same_name("stephen king", "Stephen King"));
        assert!(!same_name("Stephen King", "Stephen Fry"));
    }

    #[test]
    fn squashed_ignores_the_apostrophe_two_catalogs_disagree_on() {
        assert_eq!(
            squashed("Harry Potter and the Philosopher's Stone"),
            squashed("Harry Potter and the Philosopher\u{2019}s Stone")
        );
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn known_author_returns_bio_and_photo() {
        let info = find_author(&Client::new(), "Stephen King").await.unwrap();
        assert_eq!(info.key, "OL19981A");
        assert!(info.bio.unwrap().contains("horror"));
        assert!(info.photo_url.unwrap().starts_with("https://covers.openlibrary.org/a/id/"));
        assert!(info.work_count.unwrap() > 0);
        assert!(!info.works.is_empty());
        assert!(info.links.iter().any(|l| l.title == "Goodreads"));
    }

    /// The work search matches the author name literally, so this spelling finds
    /// a conference named after him instead. The author search has to catch it.
    #[tokio::test]
    #[ignore = "requires network"]
    async fn author_spelled_unlike_the_catalog_still_resolves() {
        let info = find_author(&Client::new(), "J. R. R. Tolkien").await.unwrap();
        assert_eq!(info.key, "OL26320A");
        assert_eq!(info.name, "J.R.R. Tolkien");
        assert!(info.series.iter().any(|s| s.name == "The Lord of the Rings"));
    }

    /// None of Sanderson's works is filed under a series entity, so only the
    /// `series:` subjects name them.
    #[tokio::test]
    #[ignore = "requires network"]
    async fn series_known_only_by_subject_still_show() {
        let info = find_author(&Client::new(), "Brandon Sanderson").await.unwrap();
        let names: Vec<&str> = info.series.iter().map(|s| s.name.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("Mistborn")), "{names:?}");
        assert!(names.iter().any(|n| n.contains("Stormlight")), "{names:?}");
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn unknown_author_returns_none() {
        assert!(find_author(&Client::new(), "Qzxwv Nonexistent Author")
            .await
            .is_none());
    }

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
