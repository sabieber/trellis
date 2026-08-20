import {ref} from 'vue';

/**
 * How the user rates a book. Both modes write the same `books.rating` column:
 * stars write 1..5, a thumb writes 1 (down) or 5 (up). Nothing is converted on a
 * switch — an existing score renders through its tendency, so the finer star
 * ratings survive a trip through thumbs mode and back.
 *
 * The mode lives on the user row, not in localStorage: a rating scale that
 * differs between the phone and the desktop is a scale nobody can read.
 * `utils/userSettings.ts` reads it and writes it.
 */
export type RatingMode = 'stars' | 'thumbs';

/** The scores the three thumbs write. The sideways one is the middle of 1..5. */
export const THUMBS_DOWN = 1;
export const THUMBS_MIDDLE = 3;
export const THUMBS_UP = 5;

export const ratingMode = ref<RatingMode>('stars');

/**
 * Which way a score leans: `1` up, `-1` down, `0` neither. A 3 sits in the
 * middle on purpose — calling it a dislike would put words in the user's mouth.
 * An unrated book (`null`, `0`) is `0` as well; the callers that must tell the
 * two apart look at the score itself.
 */
export function tendency(rating: number | null | undefined): -1 | 0 | 1 {
  if (!rating) return 0;
  if (rating >= 4) return 1;
  if (rating <= 2) return -1;
  return 0;
}
