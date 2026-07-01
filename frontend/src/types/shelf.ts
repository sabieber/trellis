export interface ShelfBook {
  id: string;
  title: string;
  author: string;
  isbn13: string | null;
  isbn10: string | null;
  google_books_id: string | null;
  open_library_id: string | null;
  added_at: string;
  rating: number | null;
}
