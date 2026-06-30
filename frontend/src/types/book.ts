export interface BookSearchResult {
  id: string
  source: 'google' | 'openlibrary'
  source_id: string
  title: string
  authors: string[]
  cover_url: string | null
  published_year: string | null
  page_count: number | null
  category: string | null
  description: string | null
  average_rating: number | null
  isbn13: string | null
  isbn10: string | null
}
