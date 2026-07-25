<template>
  <div
      class="spine"
      :class="{ 'has-cover': showCover }"
      :style="{
        width: width + 'px',
        height: actualHeight + 'px',
        background: colors.bg,
        color: colors.text,
      }"
  >
    <img
        v-if="showCover"
        class="spine-cover"
        :src="coverUrl ?? undefined"
        :alt="title"
        loading="lazy"
        @error="onError"
        @load="onLoad"
    />
    <span class="spine-title">{{ title }}</span>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {
  colorwayForTitle,
  COLORWAY_COLORS,
  spineWidth,
  spineHeightOffset,
  type Colorway
} from '@/utils/bookColorway';
import {useCoverImage} from '@/composables/useCoverImage';

const props = withDefaults(
    defineProps<{
      title: string;
      author?: string;
      colorway?: Colorway | '';
      pageCount?: number | null;
      coverUrl?: string | null;
      height?: number;
      /** Internal book UUID; when set, emit `resolve-cover` on image failure so parent can look up the real cover. */
      bookId?: string | null;
    }>(),
    {author: '', colorway: '', pageCount: null, coverUrl: null, height: 200, bookId: null},
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

const width = computed(() => spineWidth(props.title, props.pageCount));

const actualHeight = computed(() =>
    Math.round(props.height + spineHeightOffset(props.title))
);

const colors = computed(() => {
  const cw = colorwayForTitle(props.title, props.colorway);
  return COLORWAY_COLORS[cw];
});
</script>

<style scoped>
.spine {
  position: relative;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px 3px 1px 1px;
  overflow: hidden;
  cursor: pointer;
  box-shadow: inset 2px 0 0 rgb(255 255 255 / 0.08),
  inset -2px 0 0 rgb(0 0 0 / 0.18),
  0 -1px 2px rgb(0 0 0 / 0.15);
  font-family: var(--font-serif), serif;
  transition: transform 0.15s ease;
}

.spine:hover {
  transform: translateY(-6px);
}

/* Dimmed, centre-cropped front cover used as the spine texture. */
.spine-cover {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  /* Dim + slight blur so the narrow centre-crop reads as a muted spine texture
     rather than awkwardly sliced cover text, and stays consistent across books. */
  filter: brightness(0.42) saturate(0.85) blur(1.4px);
}

/* Cylindrical shading: darkened edges + off-centre highlight so the spine
   reads as slightly curved rather than a flat rectangle. */
.spine::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg,
  rgb(0 0 0 / 0.5) 0%,
  rgb(0 0 0 / 0.15) 12%,
  rgb(255 255 255 / 0.12) 38%,
  rgb(255 255 255 / 0.04) 52%,
  rgb(0 0 0 / 0.18) 80%,
  rgb(0 0 0 / 0.55) 100%);
  pointer-events: none;
}

.spine-title {
  position: relative;
  z-index: 1;
  writing-mode: vertical-rl;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-height: calc(100% - 16px);
  opacity: 0.88;
}

.spine.has-cover .spine-title {
  opacity: 0.95;
  text-shadow: 0 1px 3px rgb(0 0 0 / 0.9), 0 0 2px rgb(0 0 0 / 0.8);
}
</style>
