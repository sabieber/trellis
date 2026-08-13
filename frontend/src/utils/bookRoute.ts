import type {BookSearchResult} from '@/types/book';

/**
 * Where a book's row, tile or spine links to.
 *
 * A book the user owns has its own detail page. A catalog hit has no row in the
 * database, so it goes to the external-lookup view, which would 404 on a UUID.
 * A book with no `source` is one of the user's own — that is the shelf case.
 */
export function bookRoute(book: { id: string; source?: BookSearchResult['source'] }) {
  return {
    name: book.source && book.source !== 'library' ? 'search-detail' : 'book-detail',
    params: {id: book.id},
  };
}
