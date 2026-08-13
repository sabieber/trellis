import { apiFetch } from '@/api/client'
import type { BookSearchResult } from '@/types/book'

export const searchBooks = async (query: string): Promise<BookSearchResult[]> => {
  try {
    const response = await apiFetch(
      `/api/books/search?query=${encodeURIComponent(query)}`
    )
    if (response.ok) {
      return await response.json()
    }
    return []
  } catch {
    return []
  }
}

export const fetchTrendingBooks = async (): Promise<BookSearchResult[]> => {
  try {
    const response = await apiFetch('/api/books/trending')
    if (response.ok) {
      return await response.json()
    }
    return []
  } catch {
    return []
  }
}

export const fetchBookDetail = async (
  source: string,
  sourceId: string
): Promise<BookSearchResult | null> => {
  try {
    const response = await apiFetch(
      `/api/books/external/${source}/${encodeURIComponent(sourceId)}`
    )
    if (response.ok) {
      return await response.json()
    }
    return null
  } catch {
    return null
  }
}

// Open Library keeps every translation and reprint as a separate edition of one
// work. Only the user knows which one they own.
export const fetchEditions = async (workKey: string): Promise<BookSearchResult[]> => {
  try {
    const response = await apiFetch(`/api/books/editions/${encodeURIComponent(workKey)}`)
    if (response.ok) {
      return await response.json()
    }
    return []
  } catch {
    return []
  }
}

export const fetchSeries = async (
  key: string
): Promise<{ name: string; books: BookSearchResult[] } | null> => {
  try {
    const response = await apiFetch(`/api/series/${encodeURIComponent(key)}`)
    if (response.ok) {
      return await response.json()
    }
    return null
  } catch {
    return null
  }
}

export interface AuthorLink {
  title: string
  url: string
}

export interface AuthorSeries {
  name: string
  // Only a series Open Library files as an entity has a key, and only a key can
  // be linked to the series page.
  key: string | null
}

export interface AuthorInfo {
  key: string
  name: string
  bio: string | null
  birth_date: string | null
  death_date: string | null
  photo_url: string | null
  alternate_names: string[]
  links: AuthorLink[]
  work_count: number | null
  // The author's most-read works, minus the ones the user owns.
  works: BookSearchResult[]
  series: AuthorSeries[]
}

// Null when Open Library knows no author under that name — the screen works
// without it.
export const fetchAuthorInfo = async (name: string): Promise<AuthorInfo | null> => {
  try {
    const response = await apiFetch(`/api/authors/info?name=${encodeURIComponent(name)}`)
    if (response.ok) {
      return await response.json()
    }
    return null
  } catch {
    return null
  }
}

export const resolveGoogleId = async (bookId: string): Promise<string | null> => {
  try {
    const response = await apiFetch('/api/books/resolve-google-id', {
      method: 'POST',
      body: JSON.stringify({ book_id: bookId }),
    })
    if (response.ok) {
      const data = await response.json()
      return data.google_books_id ?? null
    }
  } catch {
  }
  return null
}
