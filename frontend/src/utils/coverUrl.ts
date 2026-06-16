export function coverUrl(googleBooksId: string | null | undefined): string | undefined {
  if (!googleBooksId) return undefined;
  return `https://books.google.com/books/content?id=${googleBooksId}&printsec=frontcover&img=1&zoom=1&source=gbs_api`;
}
