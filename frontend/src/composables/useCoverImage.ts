import { onMounted, ref, watch } from 'vue'

/**
 * Shared broken-cover detection for the two image-based book cover components
 * (BookCover, BookSpine). Google Books serves a fixed 128×170 "image not
 * available" placeholder and OpenLibrary returns a 1×1 GIF when a cover does
 * not exist; both must be treated as failures so the caller can fall back to
 * its typographic cover and — when a bookId is known — ask the parent to
 * resolve the real cover server-side.
 *
 * Pass reactive getters for the cover URL and book id plus a callback that
 * emits `resolve-cover`; get back the `imgFailed` flag and the `<img>`
 * `@error` / `@load` handlers.
 */
export function useCoverImage(
  coverUrl: () => string | null | undefined,
  bookId: () => string | null | undefined,
  onResolve: (bookId: string) => void,
) {
  const imgFailed = ref(false)
  watch(coverUrl, () => {
    imgFailed.value = false
  })

  const requestResolve = () => {
    const id = bookId()
    if (id) onResolve(id)
  }

  const onError = () => {
    imgFailed.value = true
    requestResolve()
  }

  // A book whose cover was never resolved and that has no Google thumbnail to
  // fall back on renders no `<img>` at all — so there is no load or error to
  // react to, and the two handlers above never run. Ask for it on mount
  // instead. `useBookCovers` de-duplicates the calls, and the answer is
  // persisted, so a book pays for this once and is served from the cover cache
  // from then on.
  onMounted(() => {
    if (!coverUrl()) requestResolve()
  })

  const onLoad = (e: Event) => {
    const img = e.target as HTMLImageElement
    if ((img.naturalWidth === 128 && img.naturalHeight === 170) || img.naturalWidth <= 1 || img.naturalHeight <= 1) {
      onError()
    }
  }

  return { imgFailed, onError, onLoad }
}
