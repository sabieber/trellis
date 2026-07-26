import type {Router} from 'vue-router';

// Backend collapses author-less books into this literal string (see stats.rs);
// it is not a real author and must not be linked.
export const UNKNOWN_AUTHOR = 'Unknown author';

export function isLinkableAuthor(author: string | null | undefined): author is string {
  return !!author && author !== UNKNOWN_AUTHOR;
}

// Author pages are keyed by the exact (url-encoded) author string — there is no
// author entity. Central helper so the route name/encoding lives in one place.
export function goToAuthor(router: Router, author: string): void {
  router.push({name: 'author-detail', params: {name: encodeURIComponent(author)}});
}
