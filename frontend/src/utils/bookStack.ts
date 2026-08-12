// Geometry of the pile view, where books lie flat and the reader looks at the
// stack from slightly above. `PileBook` draws the two faces; the numbers live
// here because the layout has to know how much air a book needs above it.

/** Cover width to cover height. A trade hardcover is about 2:3. */
export const DEPTH_RATIO = 0.66;

/**
 * How far the cover panel is turned away from the reader, at the bottom and at
 * the top of the viewport. 90° would be edge on.
 *
 * The eye stays level with the middle of the screen while the page scrolls, so
 * a book low on the screen shows its cover and a book at the top of the screen
 * closes to a bare spine. `ShelfPileView` reads the angle off each book's
 * position on every scroll frame, and while the page moves it swings every
 * cover further with the direction of travel — open going down the page, shut
 * going back up.
 */
export const TILT_OPEN_DEG = 80.5;
export const TILT_CLOSED_DEG = 88.5;

/**
 * Share of the cover panel that the layout keeps clear. The rest slides behind
 * the book above, which paints over it — the way the book above a book in a
 * real pile hides the back of its cover. So this is also the height of the
 * cover strip a reader sees.
 */
export const TOP_FACE_VISIBLE = 0.55;

/**
 * Viewing distance, as a multiple of the cover depth. Keeping it relative to
 * the book rather than fixed means a small book and a large one taper by the
 * same share, so the stack looks the same on a phone and on a desktop. Lower
 * values taper harder; this one takes about 14% off the far edge.
 */
export const PERSPECTIVE_DEPTHS = 6;

const RAD = Math.PI / 180;

/**
 * Height in pixels that the tilted cover panel covers on screen, above the
 * spine face, at the open angle. `PileBook` keeps `TOP_FACE_VISIBLE` of it
 * clear as its top margin and lets the rest go behind the book above.
 *
 * The panel turns around the top edge of the spine face, and the vanishing
 * point sits in the middle of that face. So the far edge of the panel lands at
 * `depth * cos(tilt)` above the turning point, `depth * sin(tilt)` behind it,
 * and the perspective divide pulls it back towards the middle of the spine.
 */
export function topFaceHeight(depth: number, thickness: number): number {
  const tilt = TILT_OPEN_DEG * RAD;
  const scale = PERSPECTIVE_DEPTHS / (PERSPECTIVE_DEPTHS + Math.sin(tilt));
  const height = scale * (depth * Math.cos(tilt) + thickness / 2) - thickness / 2;
  return Math.max(0, Math.round(height));
}
