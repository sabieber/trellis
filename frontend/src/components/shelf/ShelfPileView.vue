<template>
  <div class="book-pile">
    <div
        v-for="book in books"
        :key="book.id"
        class="pile-book"
        :style="{
          height: bookHeight(book.title) + 'px',
          width: bookWidth(book.title) + '%',
          background: bookColors(book.title).bg,
          color: bookColors(book.title).text,
        }"
        @click="$emit('viewBook', book.id)"
    >
      <span class="pile-title">{{ book.title }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  colorwayForTitle,
  COLORWAY_COLORS,
  spineWidthForTitle,
  spineHeightOffset,
  type Colorway,
} from '@/utils/bookColorway';
import type {ShelfBook} from '@/types/shelf';

const PILE_BASE_HEIGHT = 28;
const PILE_WIDTH_MIN = 85;
const PILE_WIDTH_MAX = 95;

defineProps<{
  books: ShelfBook[];
}>();

defineEmits<{
  viewBook: [id: string];
}>();

function bookHeight(title: string): number {
  return PILE_BASE_HEIGHT + Math.round(spineHeightOffset(title));
}

function bookWidth(title: string): number {
  const w = spineWidthForTitle(title);
  return PILE_WIDTH_MIN + ((w - 26) / 16) * (PILE_WIDTH_MAX - PILE_WIDTH_MIN);
}

function bookColors(title: string) {
  const cw = colorwayForTitle(title) as Colorway;
  return COLORWAY_COLORS[cw];
}
</script>

<style scoped>
.book-pile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
}

.pile-book {
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-radius: 2px;
  cursor: pointer;
  box-shadow: inset 0 2px 0 rgb(255 255 255 / 0.08),
  inset 0 -2px 0 rgb(0 0 0 / 0.18),
  0 1px 2px rgb(0 0 0 / 0.15);
  font-family: var(--font-serif), serif;
  transition: transform 0.15s ease;
  position: relative;
  overflow: hidden;
}

.pile-book:hover {
  transform: translateX(6px);
}

.pile-book::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgb(255 255 255 / 0.06), transparent 30%, transparent 70%, rgb(0 0 0 / 0.12));
  pointer-events: none;
}

.pile-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.88;
}
</style>
