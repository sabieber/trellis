<template>
  <div ref="pileRef" class="book-pile" :class="{ moving }">
    <!-- `display: contents` keeps the book itself the flex item, so the link
         adds no box of its own to the stack. -->
    <RouterLink
        v-for="(book, i) in books"
        :key="book.id"
        :to="bookRoute(book)"
        class="pile-link"
    >
      <PileBook
          :title="book.title"
          :author="book.author"
          :page-count="book.page_count"
          :width="bookWidth(book.title)"
          :stack-index="books.length - i"
          :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
          :book-id="book.id"
          :compact="compact"
          @resolve-cover="onResolveCover"
      />
    </RouterLink>
  </div>
</template>

<script setup lang="ts">
import {computed, nextTick, onMounted, onUnmounted, ref, watch} from 'vue';
import PileBook from '@/components/ui/PileBook.vue';
import {spineWidthForTitle} from '@/utils/bookColorway';
import {TILT_CLOSED_DEG, TILT_OPEN_DEG} from '@/utils/bookStack';
import {bookCoverUrl} from '@/utils/coverUrl';
import {bookRoute} from '@/utils/bookRoute';
import {useBookCovers} from '@/composables/useBookCovers';
import type {ShelfBook} from '@/types/shelf';

// Wider than this and the stack stops reading as a stack of books.
const MAX_BOOK_WIDTH = 560;
const COMPACT_BELOW = 420;

const props = defineProps<{
  books: ShelfBook[];
  containerWidth: number;
}>();

const {resolvedCoverUrl, onResolveCover} = useBookCovers();

const compact = computed(() => props.containerWidth < COMPACT_BELOW);

// Leaves room for the tilt of the widest book and for the few pixels of jitter
// that `PileBook` adds.
// The lower bound keeps the first paint sane: the container measures itself
// only once it is mounted, so the width starts at 0.
const maxWidth = computed(() =>
    Math.max(200, Math.min(props.containerWidth - 12, MAX_BOOK_WIDTH)),
);

/** Books are not all the same format, so the width varies by title. */
function bookWidth(title: string): number {
  const spread = (spineWidthForTitle(title) - 26) / 16;
  return Math.round(maxWidth.value * (0.87 + spread * 0.13));
}

// --- the turn of the covers ---
//
// A cover turns by how far its book has travelled through the viewport: it
// opens as it comes up from the bottom of the screen and closes again at the
// top. While the page is moving, every cover swings further the way inertia
// would take it — a book carried up the screen falls open, a book carried down
// falls shut — by an amount that follows the speed of the scroll. So the
// covers settle back as the page glides to rest, not after it: a settle that
// waits for the page to be fully still waits out the slow tail of a momentum
// scroll, and that wait is a pause the reader sees.
//
// A CSS view timeline could keep the resting turn off the main thread, but it
// cannot know how fast the page is moving, so everything comes from here: the
// turn and the slide of each book, and the boost, written once per frame on
// the stack itself.
//
// The page moving also draws the stack apart — see `SPREAD_PX`.
//
// Positions are cached, so a scroll frame costs one box read for the whole
// stack and one write for each book on screen.

/**
 * Frames with the stack in the same place before the page counts as at rest.
 *
 * Rest is read off the position of the stack, not off a lull in the scroll
 * events: a momentum scroll arrives in bursts, so waiting for the events to
 * stop means waiting out the longest gap between two of them, and that wait is
 * a pause the reader sees. The page itself is either still or it is not.
 */
const STILL_FRAMES = 2;

/**
 * How far apart the stack draws at full speed, top of the screen to bottom.
 * Each book slides by where it stands: the ones above the middle go up, the
 * ones below go down, so every gap in between opens by its share.
 *
 * Turning the covers alone cannot show much, because a book can never show more
 * cover than the gap to the book above it leaves — and at rest that gap is
 * already full. The gaps themselves have to open, and this is the only way to
 * open them that does not move the page under the reader: it is a transform,
 * so no book changes the space it occupies.
 */
const SPREAD_PX = 110;

// The spread follows the speed of the scroll, not just the fact of it: a slow
// browse barely stirs the stack, a flick fans it right open. Without the
// scaling, the first frame of any scroll snaps the stack apart by the full
// spread — the `.moving` class turns the transform transition off, so nothing
// eases that jump away.
//
// Speed is read in px per ms between two frames, so it means the same thing on
// a 60Hz and a 120Hz screen. It is smoothed towards its target — quick on the
// way up, slower on the way down — so the bursts a momentum scroll arrives in
// do not flutter the stack.

/** Scroll speed at which the stack is fully spread. ~1.2px/ms is a firm flick. */
const FULL_SPREAD_SPEED = 1.2;
/** Smoothing time towards a higher spread. */
const SPREAD_RISE_MS = 100;
/** Smoothing time towards a lower spread. */
const SPREAD_FALL_MS = 140;

/** Extra turn of every cover while the page moves, scaled by the spread share. */
const BOOST_OPEN_DEG = 6;
/**
 * The closing boost, on the way back up, is smaller: the closed end has little
 * headroom — a cover starts at most 8.5° short of edge-on, and `PileBook` caps
 * the turn just under 90° so no cover ever shows its back.
 */
const BOOST_SHUT_DEG = 4;
/**
 * Smoothing time for the direction of the boost alone. Its size rides the
 * spread share one to one, so the gaps and the covers open and close in step;
 * only a change of direction glides through zero here, instead of snapping the
 * covers from open to shut.
 */
const BOOST_TURN_MS = 150;

type Slot = { el: HTMLElement; offset: number; height: number };

const pileRef = ref<HTMLElement | null>(null);
const moving = ref(false);
/**
 * True while the page is on its way back up. The books then travel down the
 * screen, and the boost swings their covers shut instead of open — the way
 * inertia would take a lid hinged at the far edge.
 */
let upward = false;
let slots: Slot[] = [];
let frame = 0;
let lastBase = NaN;
let lastTime = 0;
let still = 0;
/** Share of `SPREAD_PX` in effect right now, 0 at rest to 1 at full speed. */
let spread = 0;
/** Signed full-speed degrees of the boost; smoothed so a reversal glides. */
let boostDeg = BOOST_OPEN_DEG;

function measure() {
  const root = pileRef.value;
  if (!root) return;
  const base = root.getBoundingClientRect().top;
  // The books are child components, so they are collected from the DOM rather
  // than through refs of their own. Their offset inside the stack only changes
  // when the stack is laid out again, which `remeasure` covers.
  slots = Array.from(root.querySelectorAll<HTMLElement>('.pile-book'), (el) => {
    const box = el.getBoundingClientRect();
    return {el, offset: box.top - base, height: box.height};
  });
}

function paint(knownBase?: number) {
  const root = pileRef.value;
  if (!root) return;
  const base = knownBase ?? root.getBoundingClientRect().top;
  const viewport = window.innerHeight;
  const span = TILT_CLOSED_DEG - TILT_OPEN_DEG;
  for (const slot of slots) {
    const top = base + slot.offset;
    // A book off the screen keeps the angle it left with.
    if (top < -slot.height || top > viewport) continue;
    // Half the spread above the middle of the screen, half below it.
    const place = (top + slot.height / 2) / viewport - 0.5;
    const slide = place * SPREAD_PX * spread;
    slot.el.style.setProperty('--slide', slide.toFixed(1) + 'px');
    // 0 as the book enters at the bottom of the screen, 1 as it leaves at the
    // top. Read off the slid position, so the cover turns by where the book is
    // actually drawn, not by where it would stand with the stack at rest.
    const drawnTop = top + slide;
    const progress = Math.min(1, Math.max(0, (viewport - drawnTop) / (viewport + slot.height)));
    slot.el.style.setProperty('--tilt-now', (TILT_OPEN_DEG + span * progress).toFixed(2) + 'deg');
  }
}

/** Redraws the stack once a frame for as long as the page keeps moving. */
function frameLoop(now: number) {
  frame = 0;
  const root = pileRef.value;
  if (!root) return;
  const base = root.getBoundingClientRect().top;
  // After a pause `lastTime` is old, so the speed of the first frame reads as
  // near zero — which is right, the spread of a fresh scroll starts from rest.
  const elapsed = Math.max(1, now - lastTime);
  lastTime = now;
  // `lastBase` starts as NaN, and NaN in the speed would stay in the spread
  // for good — every later frame blends towards a target computed from it.
  const speed = Math.abs(base - lastBase) / elapsed || 0;
  if (base === lastBase) {
    still += 1;
  } else {
    still = 0;
    // The pile moves down the screen when the page scrolls back up.
    if (Number.isFinite(lastBase)) upward = base > lastBase;
    lastBase = base;
  }
  // The loop keeps running while the page is still, until the spread has run
  // out: the still frames feed a speed of zero into the smoothing, so the gaps
  // close frame by frame under the loop's own hand. Handing the whole gap to
  // the CSS transition instead pops at the tail of a momentum scroll — the page
  // goes still for a few frames, the settle starts, the next burst turns the
  // transitions back off, and every half-settled book is cut to the fresh
  // frame's value in one jump.
  // The bound keeps every visible pixel of the settle in the loop's own hands:
  // at 0.01 the residuals are under half a pixel of slide and a twentieth of a
  // degree of tilt. The eased transitions that take over from here start late —
  // each book's lag, then the slow first stretch of the ease-in-out — and a
  // residual the eye can still see makes that start read as a pause.
  if (still >= STILL_FRAMES && spread < 0.01) {
    // Stopped. The watcher below hands the invisible remainder to the eased
    // transitions, after the class has come off the stack.
    spread = 0;
    moving.value = false;
    return;
  }
  const target = Math.min(1, speed / FULL_SPREAD_SPEED);
  const smoothing = target > spread ? SPREAD_RISE_MS : SPREAD_FALL_MS;
  spread += (target - spread) * (1 - Math.exp(-elapsed / smoothing));
  // The boost is the same scalar scaled to degrees, written on the stack for
  // every cover to read, so the gaps and the covers move in step. Frame by
  // frame, like the slide: a boost that waits for the page to come fully to
  // rest hangs the covers open through the slow tail of a momentum scroll.
  const degTarget = upward ? -BOOST_SHUT_DEG : BOOST_OPEN_DEG;
  boostDeg += (degTarget - boostDeg) * (1 - Math.exp(-elapsed / BOOST_TURN_MS));
  root.style.setProperty('--tilt-boost', (spread * boostDeg).toFixed(2) + 'deg');
  paint(base);
  frame = requestAnimationFrame(frameLoop);
}

function onScroll() {
  moving.value = true;
  still = 0;
  if (!frame) frame = requestAnimationFrame(frameLoop);
}

function remeasure() {
  measure();
  paint();
}

onMounted(() => {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  remeasure();
  // Captured, so a scroll inside a nested scroller is caught as well — scroll
  // events do not bubble.
  document.addEventListener('scroll', onScroll, {passive: true, capture: true});
  window.addEventListener('resize', remeasure, {passive: true});
});

onUnmounted(() => {
  document.removeEventListener('scroll', onScroll, {capture: true});
  window.removeEventListener('resize', remeasure);
  if (frame) cancelAnimationFrame(frame);
});

// The pass that draws the stack back together once the page comes to rest. It
// waits for the class, so it runs after the DOM is written and the eased
// transitions are back on — this is what carries the residuals home.
watch(moving, () => {
  if (!moving.value) pileRef.value?.style.setProperty('--tilt-boost', '0deg');
  paint();
}, {flush: 'post'});

// A new page of books arrives with no angle of its own, and a new width moves
// every book that follows it.
watch([() => props.books, () => props.containerWidth], () => nextTick(remeasure));
</script>

<style scoped>
/* Registered, so the browser knows it holds an angle and can ease it. Without
   the registration the boost still applies, it only jumps instead of easing. */
@property --tilt-boost {
  syntax: '<angle>';
  inherits: true;
  initial-value: 0deg;
}

.book-pile {
  display: flex;
  flex-direction: column;
  align-items: center;
  /* The tilted covers reach outside the row they belong to. */
  overflow: visible;
  /* Every cover reads this and swings by it while the page moves. The frame
     loop writes it on this element, scaled by the speed of the scroll and
     signed by its direction — this declaration is only the resting value for a
     stack the loop has never touched, which includes a reader who asked for no
     motion. Once the page stops, `PileBook` eases the last fraction of it home
     on each book's own clock. */
  --tilt-boost: 0deg;
}

.pile-link {
  display: contents;
}
</style>
