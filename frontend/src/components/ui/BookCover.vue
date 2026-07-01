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
    <img
        v-if="coverUrl && !imgFailed"
        class="cv-img"
        :src="coverUrl"
        :alt="title"
        loading="lazy"
        @error="imgFailed = true"
        @load="onImgLoad"
    />
    <template v-else-if="isTiny">
      <span class="cv-init">{{ (title || '?').charAt(0) }}</span>
    </template>
    <template v-else>
      <div v-if="!isSm" class="cv-author">{{ shortAuthor }}</div>
      <div class="cv-title">{{ title }}</div>
    </template>
    <div v-if="rating" class="cv-rating">
      <span class="cv-rating-star">★</span>
      <span class="cv-rating-num">{{ rating }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, ref, watch} from 'vue';

const props = withDefaults(
    defineProps<{
      title: string;
      author?: string;
      width?: number;
      colorway?: '' | 'moss' | 'clay' | 'ink' | 'plum' | 'gold' | 'char' | 'sage' | 'rust' | 'teal' | 'navy';
      coverUrl?: string | null;
      rating?: number | null;
      hoverable?: boolean;
    }>(),
    {author: '', width: 108, colorway: '', coverUrl: null, rating: null, hoverable: false},
);

const WAYS = ['moss', 'clay', 'ink', 'plum', 'gold', 'char', 'sage', 'rust', 'teal', 'navy'];

// Google Books serves a fixed 128×170 "image not available" placeholder for
// books without cover art; treat it as missing so the typographic cover shows.
const imgFailed = ref(false);
watch(() => props.coverUrl, () => {
  imgFailed.value = false;
});
const onImgLoad = (e: Event) => {
  const img = e.target as HTMLImageElement;
  if ((img.naturalWidth === 128 && img.naturalHeight === 170) || img.naturalWidth <= 1 || img.naturalHeight <= 1) imgFailed.value = true;
};

const cw = computed(() => {
  if (props.colorway) return `cv--${props.colorway}`;
  let h = 0;
  for (const ch of props.title) h = (h * 31 + ch.charCodeAt(0)) | 0;
  return `cv--${WAYS[Math.abs(h) % WAYS.length]}`;
});

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

.cv-rating-star {
  color: #d7b052;
  font-size: 9px;
}

.cv-rating-num {
  color: #f2ead8;
}
</style>
