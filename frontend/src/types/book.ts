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
  series?: SeriesRef | null
}

/// The user's own labels on a book. Genres, tags and the four "who had this
/// copy" fields are one mechanism with six headings; this discriminator is all
/// that separates them.
export type LabelKind =
    'genre' | 'tag' | 'received_from' | 'given_to' | 'borrowed_from' | 'borrowed_to'

/// The kinds whose labels are people. They draw from one suggestion pool.
export const PERSON_KINDS = ['received_from', 'given_to', 'borrowed_from', 'borrowed_to'] as const

/// Which pool a kind's suggestions come from — the key the backend sends them under.
export type SuggestionPool = 'genre' | 'tag' | 'person'

/// A note the user wrote about a book. `page` is optional: a note about the
/// book as a whole carries none.
export interface BookNote {
  id: string
  text: string
  page: number | null
  created_at: string
  updated_at: string
}

export interface SeriesRef {
  key: string
  name: string
  position: string | null
}
