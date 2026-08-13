import type {BookSearchResult} from '@/types/book';

export interface ShelfBook {
  id: string;
  /**
   * Set only on a catalog hit the shelf layouts render (see `asShelfBook`).
   * A book of the user's own leaves it out.
   */
  source?: BookSearchResult['source'];
  title: string;
  author: string;
  isbn13: string | null;
  isbn10: string | null;
  google_books_id: string | null;
  open_library_id: string | null;
  added_at: string;
  rating: number | null;
  cover_url: string | null;
  page_count: number | null;
}
