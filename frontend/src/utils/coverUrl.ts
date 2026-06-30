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
  google_books_id?: string | null;
  isbn13?: string | null;
  isbn10?: string | null;
}): string | undefined {
  return googleCoverUrl(book.google_books_id) ?? openLibraryCoverUrl(book.isbn13, book.isbn10);
}
