export interface BookSearchResult {
  id: string
  source: 'google' | 'openlibrary' | 'library'
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
  // Detail-only fields, populated by the external/{source}/{id} endpoint.
  subtitle?: string | null
  publisher?: string | null
  published_date?: string | null
  language?: string | null
  categories?: string[]
  ratings_count?: number | null
  info_link?: string | null
}
