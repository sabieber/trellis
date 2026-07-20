import { ref } from 'vue'
import { resolveBookCoverUrl } from '@/utils/coverUrl'

/**
 * Tracks resolved cover URLs and provides a handler for the
 * `resolve-cover` events emitted by `BookCover` components.
 *
 * Call `onResolveCover` when a BookCover emits its `resolve-cover`
 * event. The handler is de-duplicated so the same book is never
 * resolved twice in one view lifecycle.
 *
 * Use `resolvedCoverUrl(bookId, fallbackUrl)` to look up the best
 * available cover URL: the server-resolved one takes priority, then
 * the given fallback (e.g. from ISBN-based generation).
 */
export function useBookCovers() {
  const resolvedUrls = ref<Record<string, string>>({})
  const pending = new Set<string>()

  const onResolveCover = async (bookId: string) => {
    if (pending.has(bookId) || resolvedUrls.value[bookId]) return
    pending.add(bookId)
    try {
      const url = await resolveBookCoverUrl(bookId)
      if (url) {
        resolvedUrls.value[bookId] = url
      }
    } finally {
      pending.delete(bookId)
    }
  }

  const resolvedCoverUrl = (
    bookId: string | null | undefined,
    fallbackUrl: string | undefined,
  ): string | undefined => {
    if (bookId && resolvedUrls.value[bookId]) {
      return resolvedUrls.value[bookId]
    }
    return fallbackUrl
  }

  return { resolvedUrls, onResolveCover, resolvedCoverUrl }
}
