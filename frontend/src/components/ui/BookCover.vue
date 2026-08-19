<!-- Typographic placeholder cover.
     Three sizes by width: normal (author + title), sm (≤90px: title only),
     tiny (≤52px: single initial). Colorway can be passed or is hashed
     deterministically from the title so a book always gets the same color.
     When real cover art exists, pass `coverUrl` — the image replaces the
     typographic content but keeps the aspect-ratio, radius, and shadow. -->
<template>
  <div
      class="cover"
      :class="[cw, { sm: isSm, tiny: isTiny, 'hoverable-card': hoverable }]"
      :style="{ width: width + 'px' }"
  >
    <!-- The typographic cover is always rendered, and the art paints over it —
         `.cv-img` is absolutely positioned, so it takes part in no layout. That
         is what keeps a tile still while it loads: a book whose cover has not
         been resolved yet draws its lettering immediately and keeps it, and the
         image, when it arrives, covers that up without moving anything. Swapping
         the two with v-if/v-else instead made every such tile change its content
         once the cover resolved, and flash empty whenever one failed to load. -->
    <template v-if="isTiny">
      <span class="cv-init">{{ (title || '?').charAt(0) }}</span>
    </template>
    <template v-else>
      <div v-if="!isSm" class="cv-author">{{ shortAuthor }}</div>
      <div class="cv-title">{{ title }}</div>
    </template>
    <!-- Decorative, and `alt` must stay empty: the lettering underneath already
         carries the title as real text, so alt text here would both announce it
         twice and — because Firefox paints the alt text of a broken or pending
         image inside the image's own box — draw the title a second time across
         the top of the tile until the image resolves or fails. -->
    <img
        v-if="showingArt"
        class="cv-img"
        :src="coverUrl ?? undefined"
        alt=""
        loading="lazy"
        @error="onError"
        @load="onLoad"
    />
    <!-- "This book has notes" — a mark, not a count: the number is only interesting once you open the book. -->
    <div v-if="hasNote" class="cv-note">
      <StickyNoteIcon class="cv-note-icon" fill="color-mix(in srgb, currentColor 50%, transparent)"/>
    </div>
    <div v-if="rating" class="cv-rating">
      <FlowerIcon class="cv-rating-icon" fill="color-mix(in srgb, currentColor 50%, transparent)"/>
      <span class="cv-rating-num">{{ rating }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {FlowerIcon, StickyNoteIcon} from '@lucide/vue';
import {useCoverImage} from '@/composables/useCoverImage';

const props = withDefaults(
    defineProps<{
      title: string;
      author?: string;
      width?: number;
      colorway?: '' | 'moss' | 'clay' | 'ink' | 'plum' | 'gold' | 'char' | 'sage' | 'rust' | 'teal' | 'navy';
      coverUrl?: string | null;
      rating?: number | null;
      /** Marks the cover with a note badge when the book carries at least one. */
      hasNote?: boolean;
      hoverable?: boolean;
      /** Internal book UUID; when set, emit `resolve-cover` on image failure so parent can look up the real cover. */
      bookId?: string | null;
    }>(),
    {author: '', width: 108, colorway: '', coverUrl: null, rating: null, hasNote: false, hoverable: false, bookId: null},
);

const emit = defineEmits<{
  /** Emitted when the cover image fails to load and we have a bookId to resolve. */
  'resolve-cover': [bookId: string];
}>();

const WAYS = ['moss', 'clay', 'ink', 'plum', 'gold', 'char', 'sage', 'rust', 'teal', 'navy'];

const {imgFailed, onError, onLoad} = useCoverImage(
    () => props.coverUrl,
    () => props.bookId,
    (id) => emit('resolve-cover', id),
);

const cw = computed(() => {
  if (props.colorway) return `cv--${props.colorway}`;
  let h = 0;
  for (const ch of props.title) h = (h * 31 + ch.charCodeAt(0)) | 0;
  return `cv--${WAYS[Math.abs(h) % WAYS.length]}`;
});

const showingArt = computed(() => Boolean(props.coverUrl) && !imgFailed.value);

const isTiny = computed(() => props.width <= 52);
const isSm = computed(() => !isTiny.value && props.width <= 90);

// "Ursula K. Le Guin" → "U. K. L. Guin"-style compression
const shortAuthor = computed(() => {
  const a = props.author;
  if (!a) return '';
  const parts = a.split(' ');
  if (parts.length < 2) return a;
  const last = parts.pop();
  return parts.map((p) => p[0] + '.').join(' ') + ' ' + last;
});
</script>

<style scoped>
.cover {
  position: relative;
  aspect-ratio: 2/3;
  border-radius: 5px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  flex: none;
  padding: 10px 9px;
  color: #f2ead8;
  box-shadow: 0 1px 0 rgb(255 255 255 / 0.05) inset,
  0 1px 2px rgb(0 0 0 / 0.4), 0 8px 24px rgb(0 0 0 / 0.28);
  font-family: var(--font-serif), serif;
  isolation: isolate;
}

.cover::after { /* spine shading + paper sheen */
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  background: linear-gradient(105deg, rgb(255 255 255 / 0.10), transparent 22%),
  linear-gradient(0deg, rgb(0 0 0 / 0.32), transparent 55%);
}

.cv-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cv-author {
  font-family: var(--font-sans), sans-serif;
  font-size: 8px;
  font-weight: 600;
  letter-spacing: 0.13em;
  text-transform: uppercase;
  opacity: 0.82;
}

.cv-title {
  margin-top: auto;
  font-weight: 600;
  font-size: 16px;
  line-height: 1.06;
  letter-spacing: -0.01em;
  text-wrap: balance;
}

.cover.sm {
  padding: 9px 8px;
}

.cover.sm .cv-title {
  font-size: 12px;
}

.cover.tiny {
  align-items: center;
  justify-content: center;
  padding: 4px;
}

.cv-init {
  margin: auto;
  font-weight: 600;
  font-size: 17px;
  opacity: 0.9;
}

/* muted, on-brand colorways */
.cv--moss {
  background: #33402a;
  color: #e7e2c8;
}

.cv--clay {
  background: #5a3b2c;
  color: #f1e2d0;
}

.cv--ink {
  background: #23314a;
  color: #e3e7f0;
}

.cv--plum {
  background: #412a3e;
  color: #efdfe9;
}

.cv--gold {
  background: #5c4a23;
  color: #f6eccf;
}

.cv--char {
  background: #262420;
  color: #e7ddc8;
}

.cv--sage {
  background: #3c4636;
  color: #e9ecd9;
}

.cv--rust {
  background: #532a23;
  color: #f3ddcf;
}

.cv--teal {
  background: #1f3d3a;
  color: #dcece6;
}

.cv--navy {
  background: #1d2740;
  color: #dfe3ee;
}

/* Bottom left, mirroring the rating badge on the right, so both can sit on
   one cover without touching. */
.cv-note {
  position: absolute;
  left: 4px;
  bottom: 4px;
  display: flex;
  align-items: center;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  border-radius: 4px;
  padding: 3px;
  line-height: 1;
  z-index: 2;
}

.cv-note-icon {
  color: var(--color-green-soft);
  width: 10px;
  height: 10px;
}

.cv-rating {
  position: absolute;
  right: 4px;
  bottom: 4px;
  display: flex;
  align-items: center;
  gap: 1px;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  border-radius: 4px;
  padding: 2px 5px;
  font-family: var(--font-sans), sans-serif;
  font-size: 10px;
  font-weight: 600;
  line-height: 1;
  z-index: 2;
}

.cv-rating-icon {
  color: var(--color-gold);
  width: 10px;
  height: 10px;
}

.cv-rating-num {
  color: #f2ead8;
}
</style>
