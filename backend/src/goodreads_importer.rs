use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::io::Read;

/// Parses a GoodReads date string (`YYYY/MM/DD`) into a `NaiveDate`.
///
/// Returns `None` for empty or malformed input.
pub fn parse_goodreads_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y/%m/%d").ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BookRecord {
    #[serde(rename = "Book Id")]
    #[allow(dead_code)]
    pub book_id: String,
    pub title: String,
    pub author: String,
    #[serde(rename = "ISBN")]
    pub isbn: String,
    #[serde(rename = "ISBN13")]
    pub isbn13: String,
    #[allow(dead_code)]
    #[serde(rename = "My Rating")]
    pub my_rating: Option<u8>,
    #[allow(dead_code)]
    pub publisher: String,
    #[allow(dead_code)]
    pub binding: String,
    #[serde(rename = "Number of Pages")]
    pub number_of_pages: Option<u32>,
    #[allow(dead_code)]
    #[serde(rename = "Year Published")]
    pub year_published: Option<u16>,
    #[serde(rename = "Date Read")]
    pub date_read: Option<String>,
    #[serde(rename = "Date Added")]
    pub date_added: String,
    pub bookshelves: String,
    #[serde(rename = "Exclusive Shelf")]
    pub exclusive_shelf: String,
    #[allow(dead_code)]
    #[serde(rename = "My Review")]
    pub my_review: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "Private Notes")]
    pub private_notes: Option<String>,
    #[serde(rename = "Read Count")]
    pub read_count: u8,
    #[allow(dead_code)]
    #[serde(rename = "Owned Copies")]
    pub owned_copies: u8,
}

impl BookRecord {
    pub fn from_reader(data: impl Read) -> Result<Vec<BookRecord>, Box<dyn std::error::Error>> {
        let mut rdr = ReaderBuilder::new().from_reader(data);
        let mut records = Vec::new();
        for result in rdr.deserialize() {
            let record: BookRecord = result?;
            records.push(record);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_goodreads_date() {
        assert_eq!(
            parse_goodreads_date("2025/02/20"),
            chrono::NaiveDate::from_ymd_opt(2025, 2, 20)
        );
    }

    #[test]
    fn returns_none_for_empty_string() {
        assert_eq!(parse_goodreads_date(""), None);
    }

    #[test]
    fn returns_none_for_garbage() {
        assert_eq!(parse_goodreads_date("not-a-date"), None);
    }
}
