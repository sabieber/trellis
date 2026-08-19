<!-- One book lying flat, seen slightly from above: the spine faces the reader and
     the front cover recedes as the top surface. Both faces are real CSS 3D — the
     cover panel is rotated back around the book's top edge, so the perspective
     narrows its far edge into a trapezoid on its own.

     The cover art is rotated a quarter turn because the book lies on its back:
     the cover's spine edge is the edge nearest the reader. The same art takes
     the same turn, dimmed and blurred, as the texture of the spine — the way
     `BookSpine` does it on the shelf — so the two faces keep one colour. -->
<template>
  <div
      class="pile-book"
      :class="{ 'has-cover': showCover }"
      :style="{
        '--w': width + 'px',
        '--thick': thickness + 'px',
        '--depth': depth + 'px',
        '--top-h': topHeight + 'px',
        '--jx': jitter + 'px',
        '--rot': rotation + 'deg',
        '--z': String(stackIndex),
        '--lag': lag + 'ms',
        '--tilt-open': TILT_OPEN_DEG + 'deg',
        '--persp': depth * PERSPECTIVE_DEPTHS + 'px',
        background: colors.bg,
        color: colors.text,
      }"
  >
    <div class="pb-top">
      <!-- Decorative, and `alt` must stay empty: `.pb-title` below always
           renders the title as real text. A non-empty `alt` would announce it
           twice, and Firefox paints the alt text of a broken or pending image
           inside the image's box — drawing the title across the cover panel
           until the art resolves or fails. -->
      <img
          v-if="showCover"
          class="pb-top-art"
          :src="coverUrl ?? undefined"
          alt=""
          loading="lazy"
          @error="onError"
          @load="onLoad"
      />
      <div v-else class="pb-top-blank"></div>
    </div>
    <!-- Decorative, like the cover panel above: `.pb-title` carries the title. -->
    <img
        v-if="showCover"
        class="pb-spine-art"
        :src="coverUrl ?? undefined"
        alt=""
        loading="lazy"
    />
    <span v-if="!compact && author" class="pb-author">{{ author }}</span>
    <span class="pb-title">{{ title }}</span>
    <FlowerIcon v-if="!compact" class="pb-mark" aria-hidden="true"/>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {FlowerIcon} from '@lucide/vue';
import {
  colorwayForTitle,
  COLORWAY_COLORS,
  spineWidth,
  type Colorway,
} from '@/utils/bookColorway';
import {useCoverImage} from '@/composables/useCoverImage';
import {
  DEPTH_RATIO,
  PERSPECTIVE_DEPTHS,
  TILT_OPEN_DEG,
  TOP_FACE_VISIBLE,
  topFaceHeight,
} from '@/utils/bookStack';

const props = withDefaults(
    defineProps<{
      title: string;
      /** Width of the spine face; the book's height when it stands up. */
      width: number;
      /**
       * Paint order in the stack, counting down from the first book. A book
       * has to paint over the one under it, so that its own body hides the
       * back of that book's cover — the whole stack rests on this.
       */
      stackIndex: number;
      author?: string;
      colorway?: Colorway | '';
      pageCount?: number | null;
      coverUrl?: string | null;
      /** Narrow container: drop the author and the end mark, keep the title. */
      compact?: boolean;
      /** Internal book UUID; when set, emit `resolve-cover` on image failure so parent can look up the real cover. */
      bookId?: string | null;
    }>(),
    {author: '', colorway: '', pageCount: null, coverUrl: null, compact: false, bookId: null},
);

const emit = defineEmits<{
  'resolve-cover': [bookId: string];
}>();

const {imgFailed, onError, onLoad} = useCoverImage(
    () => props.coverUrl,
    () => props.bookId,
    (id) => emit('resolve-cover', id),
);

const showCover = computed(() => props.coverUrl && !imgFailed.value);

// Page count drives the thickness, the same way it drives the spine width when
// the book stands on a shelf.
const thickness = computed(() => spineWidth(props.title, props.pageCount));

const depth = computed(() => Math.round(props.width * DEPTH_RATIO));

const topHeight = computed(() =>
    Math.round(topFaceHeight(depth.value, thickness.value) * TOP_FACE_VISIBLE),
);

// A stack that is machine straight looks like a chart. These two put every book
// a few pixels off true, always by the same amount for the same title.
const jitter = computed(() => hash(props.title, 53) % 11 - 5);
const rotation = computed(() => ((hash(props.title, 59) % 9) - 4) / 10);

// How late this book is to open and to fall shut again. Books that all move on
// the same tick read as one board rather than as a stack of separate things.
const lag = computed(() => hash(props.title, 61) % 110);

const colors = computed(() => COLORWAY_COLORS[colorwayForTitle(props.title, props.colorway)]);

function hash(title: string, seed: number): number {
  let h = 0;
  for (const ch of title) h = (h * seed + ch.charCodeAt(0)) | 0;
  return Math.abs(h);
}
</script>

<style scoped>
.pile-book {
  position: relative;
  flex: none;
  display: flex;
  align-items: center;
  gap: 12px;
  width: var(--w);
  height: var(--thick);
  /* Books in a pile touch. `--top-h` is the part of the cover panel that stays
     in the open; the rest of it runs behind the book above, which paints over
     it. The 3px is the daylight between two volumes. */
  margin-top: calc(var(--top-h) + 3px);
  /* Counting down the stack, so each book paints over the one below it: its
     body hides the back of that book's cover, and its shadow falls on it. */
  z-index: var(--z);
  padding: 0 clamp(10px, 4%, 30px);
  border-radius: 1px 1px 3px 3px;
  /* Each book gets its own vanishing point, so the first and the last book of a
     long stack are drawn at the same angle. */
  perspective: var(--persp);
  /* `--slide` draws the stack apart while the page moves — see `ShelfPileView`.
     It is what lets the covers show anything: turning a cover cannot help if
     the book above leaves it no room. */
  transform: translateX(var(--jx)) translateY(var(--slide, 0px)) rotate(var(--rot));
  /* Line 1 is the lit edge of the cover board where the two faces meet —
     without it the cover and the spine melt into one surface. The last two are
     the shadow: a tight one at the contact line and a wide, soft one under it,
     both landing on the cover of the book below. */
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.3),
  inset 0 -2px 3px rgb(0 0 0 / 0.45),
  inset 2px 0 4px rgb(0 0 0 / 0.3),
  inset -2px 0 4px rgb(0 0 0 / 0.3),
  0 2px 3px -1px rgb(0 0 0 / 0.55),
  0 16px 22px -10px rgb(0 0 0 / 0.75);
  font-family: var(--font-serif), serif;
  text-decoration: none;
  /* Transform only, plus the boost. A box-shadow or opacity transition repaints
     this book and its blurred cover art on every frame, and the browser
     re-rasterizes the neighbours in the same 3D group with it — which shows up
     as a flicker across the stack. The other hover changes therefore switch at
     once.

     While the page moves, the frame loop in `ShelfPileView` drives both the
     slide and `--tilt-boost` itself, already smoothed — the rule below turns
     these transitions off so every write lands on the frame it was made for.
     What is left to them is the tail: once the page stops, the last couple of
     pixels and the last fraction of the boost ease home from here, each book
     after a lag of its own, on an ease-in-out — a book so near rest has to
     leave gently and arrive gently, and books that all start on the same frame
     read as one board rather than as a stack. Both run the same clock, so the
     gaps and the covers arrive together. */
  transition: transform 400ms cubic-bezier(0.65, 0, 0.35, 1) var(--lag),
  --tilt-boost 400ms cubic-bezier(0.65, 0, 0.35, 1) var(--lag);
}

/* While the page moves, the stack answers at once: the slide and the boost are
   rewritten every frame and have to land on the frame they are written for. An
   eased transition here would be retargeted by every write and, with its lag,
   would hold the value back for good. */
.book-pile.moving .pile-book {
  transition: transform 0s, --tilt-boost 0s;
}

/* Cylindrical shading, as on the shelf: dark at both edges with the highlight
   off centre, so the spine reads as rounded and not as a flat bar. Here the
   book lies down, so the curve runs from the top edge to the bottom one. */
.pile-book::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: linear-gradient(180deg,
  rgb(0 0 0 / 0.32) 0%,
  rgb(0 0 0 / 0.06) 14%,
  rgb(255 255 255 / 0.13) 38%,
  rgb(255 255 255 / 0.04) 54%,
  rgb(0 0 0 / 0.14) 80%,
  rgb(0 0 0 / 0.42) 100%);
  pointer-events: none;
}

/* Dimmed, centre-cropped cover used as the spine texture, so a book's spine
   carries the colour of its own cover instead of a colour drawn from its
   title. It takes the same quarter turn as the cover above it, because the
   book lies on its back: the strip that wraps onto the spine runs along the
   cover's own vertical axis. Sized before the turn, so width is the height of
   the face and height is its width. Clipped because the blur would otherwise
   bleed past the edges. */
.pb-spine-art {
  position: absolute;
  top: 50%;
  left: 50%;
  z-index: -1;
  width: var(--thick);
  height: var(--w);
  margin: calc(var(--w) / -2) 0 0 calc(var(--thick) / -2);
  transform: rotate(-90deg);
  object-fit: cover;
  clip-path: inset(0 round 2px);
  filter: brightness(0.6) saturate(1.05) blur(1.4px);
}

/* --- the front cover, lying flat --- */
.pb-top {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 100%;
  height: var(--depth);
  overflow: hidden;
  border-radius: 3px 3px 0 0;
  background: inherit;
  transform-origin: bottom center;
  /* The eye stays level with the middle of the screen, so a book opens as it
     comes up from the bottom of it and closes again at the top. `--tilt-now`
     carries that angle, and `ShelfPileView` writes it on every scroll frame.
     `--tilt-boost` swings the cover further while the page moves — open on the
     way down the page, shut on the way back up; the same view eases it in and
     out. Both fall back to the open angle, which is what a book shows when
     nothing runs — a reader who asked for no motion, or a book that has not
     been on screen yet.

     Capped just short of 90°: a cover already nearly shut plus the closing
     boost would turn past edge-on and show the reader its back. */
  transform: rotateX(min(calc(var(--tilt-now, var(--tilt-open)) - var(--tilt-boost, 0deg)), 89deg));
}

/* The shading of the cover, thinned out on hover so the book brightens. It
   starts at the near edge with the joint: the channel pressed into the cover
   where the board is hinged to the spine. That channel belongs to the cover,
   not to the spine, so it is drawn here, against the spine's own lit edge on
   the other side of the line.

   No hard stops in it. A groove is a surface that falls away and comes back,
   so the light on it ramps: darkest in the channel at the hinge line, back up
   to a highlight where the board rises out of it, then the cover. Hard stops
   make a painted stripe instead. It is stated in the panel's own pixels, which
   the tilt then flattens — 18px of cover is a groove of a few millimetres. */
.pb-top::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(0deg,
  rgb(0 0 0 / 0.6) 0,
  rgb(0 0 0 / 0.34) 7px,
  rgb(255 255 255 / 0.15) 18px,
  rgb(255 255 255 / 0.05) 22%,
  rgb(0 0 0 / 0.14) 65%,
  rgb(0 0 0 / 0.45) 100%);
}

.pb-top-art {
  position: absolute;
  top: 50%;
  left: 50%;
  width: var(--depth);
  height: var(--w);
  margin: calc(var(--w) / -2) 0 0 calc(var(--depth) / -2);
  /* The book lies on its back, so the cover's spine edge points at the reader. */
  transform: rotate(-90deg);
  object-fit: cover;
}

/* Without cover art the panel is a plain slab, so it gets the sheen and the
   blind-stamped border of a cloth board instead. */
.pb-top-blank {
  position: absolute;
  inset: 0;
  border: 1px solid rgb(255 255 255 / 0.06);
  background: linear-gradient(115deg,
  rgb(255 255 255 / 0.08) 0%,
  transparent 30%,
  transparent 70%,
  rgb(0 0 0 / 0.12) 100%);
}

/* --- spine lettering --- */
.pb-author,
.pb-title {
  position: relative;
  font-size: clamp(9px, calc(var(--thick) * 0.32), 15px);
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.9;
  text-shadow: 0 1px 2px rgb(0 0 0 / 0.5);
}

.pb-author {
  flex: 0 1 auto;
  max-width: 34%;
  font-style: italic;
  opacity: 0.72;
}

.pb-title {
  flex: 1 1 auto;
  text-align: center;
  font-weight: 600;
  letter-spacing: 0.01em;
}

/* Lettering over cover art needs to hold its own against a busy texture. */
.pile-book.has-cover .pb-author,
.pile-book.has-cover .pb-title {
  opacity: 0.95;
  text-shadow: 0 1px 3px rgb(0 0 0 / 0.9), 0 0 2px rgb(0 0 0 / 0.8);
}

.pb-mark {
  position: relative;
  flex: none;
  width: clamp(10px, calc(var(--thick) * 0.4), 18px);
  height: clamp(10px, calc(var(--thick) * 0.4), 18px);
  opacity: 0.5;
}

/* Hover draws the book out of the stack, towards the reader: it grows, and it
   travels down the screen, which is the direction the reader sits in. Its cover
   slides out from under the book above as it comes, so more of the cover shows
   the further it is out.
   It keeps its own `--z`. Lifting it over the stack would show the part of its
   cover that belongs behind the book above, drawn across that book's spine —
   and the book above still lies on top of this one, however far it is pulled
   out. Everything the growth adds at the top therefore goes out of sight
   behind that book, and only the books below it are covered, which is where
   the reader is looking from. */
.pile-book:hover {
  transform: translateX(var(--jx)) translateY(calc(var(--slide, 0px) + 5px)) rotate(var(--rot)) scale(1.05);
  transition: transform 0.22s ease, --tilt-boost 400ms ease var(--lag);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.38),
  inset 0 -2px 3px rgb(0 0 0 / 0.45),
  inset 2px 0 4px rgb(0 0 0 / 0.3),
  inset -2px 0 4px rgb(0 0 0 / 0.3),
  0 6px 8px -3px rgb(0 0 0 / 0.6),
  0 26px 32px -14px rgb(0 0 0 / 0.85);
}

.pile-book:hover .pb-top::after {
  opacity: 0.6;
}

@media (prefers-reduced-motion: reduce) {
  .pile-book,
  .book-pile.moving .pile-book {
    transition: none;
  }
}
</style>
