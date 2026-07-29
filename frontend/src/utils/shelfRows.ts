import {spineWidth} from '@/utils/bookColorway'

export interface ShelfRow<T> {
    books: T[]
    /** Index of the row's first book in the flat list — used for staggered animations. */
    start: number
}

/**
 * Packs books into shelf rows that fit `width`, using each book's rendered
 * spine width. A non-positive width (before the container is measured) puts
 * everything in one row.
 */
export function packShelfRows<T extends { title: string; page_count: number | null }>(
    books: T[],
    width: number,
    gap = 3,
): ShelfRow<T>[] {
    if (width <= 0 || !books.length) return [{books, start: 0}]
    const rows: ShelfRow<T>[] = []
    let row: T[] = []
    let start = 0
    let rowWidth = 0
    books.forEach((book, i) => {
        const bookWidth = spineWidth(book.title, book.page_count)
        const needed = row.length === 0 ? bookWidth : bookWidth + gap
        if (rowWidth + needed > width && row.length > 0) {
            rows.push({books: row, start})
            row = [book]
            start = i
            rowWidth = bookWidth
        } else {
            row.push(book)
            rowWidth += needed
        }
    })
    if (row.length) rows.push({books: row, start})
    return rows
}
