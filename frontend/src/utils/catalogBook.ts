import type {BookSearchResult} from '@/types/book';
import type {ShelfBook} from '@/types/shelf';

/**
 * A catalog hit in the shape the shelf layouts read, so a list of search
 * results renders as a grid, a shelf or a pile like the user's own books.
 *
 * `source` survives the mapping and sends the link to the external-lookup view
 * (see `bookRoute`).
 */
export function asShelfBook(book: BookSearchResult): ShelfBook {
  return {
    id: book.id,
    source: book.source,
    title: book.title,
    author: book.authors?.join(', ') ?? '',
    isbn13: book.isbn13,
    isbn10: book.isbn10,
    google_books_id: null,
    open_library_id: null,
    added_at: '',
    // ponytail: `average_rating` is the catalog's rating, not the user's, and
    // the cover badge means "you rated this". Drop it rather than mislead.
    rating: null,
    cover_url: book.cover_url,
    page_count: book.page_count,
  };
}
