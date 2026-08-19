import { apiFetch } from '@/api/client'

export function bookCoverUrl(book: { cover_url?: string | null }): string | undefined {
  // The only cover URL the frontend ever builds for one of the user's own
  // books. The backend sends `cover_url` as its own cover-proxy URL
  // (`/api/books/{id}/cover?v=…`): served from the cover cache, and kept by the
  // browser for a year.
  //
  // Nothing is hotlinked here any more, not even a Google thumbnail. A book
  // with no `cover_url` yet has never been resolved, and `useCoverImage` asks
  // the backend to resolve it on mount — once, into the cache — rather than the
  // frontend guessing a catalog URL that stays slow (Open Library) or silently
  // renders Google's "image not available" placeholder.
  return book.cover_url ?? undefined;
}

/**
 * Calls the backend to resolve a book's cover URL using external APIs
 * (Google Books, Open Library work lookup, ISBN fallback).
 * The resolved URL is cached server-side so subsequent calls are instant.
 *
 * Returns the resolved URL or `null` if resolution failed.
 */
export async function resolveBookCoverUrl(bookId: string): Promise<string | null> {
  try {
    const response = await apiFetch('/api/books/resolve-cover', {
      method: 'POST',
      body: JSON.stringify({ book_id: bookId }),
    })
    if (response.ok) {
      const data = await response.json()
      return data.cover_url ?? null
    }
  } catch {
    // Network error — swallow silently, the fallback cover will be shown.
  }
  return null
}
