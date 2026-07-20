import { apiFetch } from '@/api/client'

export function googleCoverUrl(googleBooksId: string | null | undefined): string | undefined {
  if (!googleBooksId) return undefined;
  return `https://books.google.com/books/content?id=${googleBooksId}&printsec=frontcover&img=1&zoom=1&source=gbs_api`;
}

export function openLibraryCoverUrl(isbn13: string | null | undefined, isbn10?: string | null): string | undefined {
  const isbn = isbn13 || isbn10;
  if (!isbn) return undefined;
  return `https://covers.openlibrary.org/b/isbn/${isbn}-M.jpg`;
}

export function bookCoverUrl(book: {
  cover_url?: string | null;
  google_books_id?: string | null;
  isbn13?: string | null;
  isbn10?: string | null;
}): string | undefined {
  // Prefer the server-resolved and cached cover URL. Falls back to
  // generating URLs from identifiers (Google Books thumbnail, then the
  // ISBN-based Open Library cover which may fail when the cover lives on
  // the work rather than the edition).
  if (book.cover_url) return book.cover_url;
  return googleCoverUrl(book.google_books_id) ?? openLibraryCoverUrl(book.isbn13, book.isbn10);
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
