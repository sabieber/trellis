import { apiFetch } from '@/api/client'
import type { BookSearchResult } from '@/types/book'

export const searchBooks = async (query: string): Promise<BookSearchResult[]> => {
  try {
    const response = await apiFetch(
      `/api/books/search?q=${encodeURIComponent(query)}`
    )
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
