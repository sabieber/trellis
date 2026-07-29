<template>
  <div class="flex flex-col" :class="{ 'has-pick': !!pickedId }">
    <div v-for="(row, ri) in shelfRows" :key="ri" class="shelf-row">
      <div class="shelf-books">
        <BookSpine
            v-for="(book, bi) in row.books"
            :key="book.id"
            :title="book.title"
            :author="book.author"
            :page-count="book.page_count"
            :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
            :book-id="book.id"
            :height="spineHeight"
            :class="{ gust, picked: book.id === pickedId, taken: book.id === pickedId && pickedTaken }"
            :style="{ '--i': row.start + bi }"
            @click="$emit('viewBook', book.id)"
            @resolve-cover="onResolveCover"
        />
      </div>
      <div class="shelf-board"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import BookSpine from '@/components/ui/BookSpine.vue';
import {bookCoverUrl} from '@/utils/coverUrl';
import {packShelfRows} from '@/utils/shelfRows';
import {useBookCovers} from '@/composables/useBookCovers';
import type {ShelfBook} from '@/types/shelf';

const {resolvedCoverUrl, onResolveCover} = useBookCovers();

const props = withDefaults(
    defineProps<{
      books: ShelfBook[];
      spineHeight: number;
      containerWidth: number;
      /** Wind pass: spines sway one after another. */
      gust?: boolean;
      /** Highlights one spine and dims the rest. */
      pickedId?: string | null;
      /** The picked book has left the shelf — its spine slides out and the gap closes. */
      pickedTaken?: boolean;
    }>(),
    {gust: false, pickedId: null, pickedTaken: false},
);

defineEmits<{
  viewBook: [id: string];
}>();

const shelfRows = computed(() => packShelfRows(props.books, props.containerWidth));
</script>

<style scoped>
.shelf-row {
  margin-bottom: 24px;
}

.shelf-books {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  padding: 0 8px;
}

.shelf-board {
  height: 16px;
  margin-top: -1px;
  background: linear-gradient(180deg, #7a5a3a 0%, #6b4c30 20%, #5a3b2c 50%, #4a2e20 100%);
  border-radius: 0 0 3px 3px;
  box-shadow: inset 0 2px 0 rgb(255 255 255 / 0.08),
  0 4px 12px rgb(0 0 0 / 0.4),
  0 2px 4px rgb(0 0 0 / 0.3);
}

/* --- wind pass (random picker) ---
   Timed so the last spine (--i 15) lands back at rest just before the picker
   switches phase: 15 × 30ms + 2 × 560ms = 1570ms < GUST_MS. Otherwise the
   class is pulled mid-sway and the whole row snaps. */
:deep(.spine.gust) {
  transform-origin: bottom center;
  animation: sway 560ms ease-in-out calc(var(--i) * 30ms) 2;
}

@keyframes sway {
  0%, 100% {
    transform: none;
  }
  35% {
    transform: rotate(-3.2deg) translateY(-3px);
  }
  70% {
    transform: rotate(1.4deg);
  }
}

/* Everything fades back except the book the breeze settled on. */
.has-pick :deep(.spine) {
  transition: opacity 450ms ease, filter 450ms ease, box-shadow 450ms ease,
  transform 450ms cubic-bezier(0.2, 0.9, 0.25, 1),
  width 420ms cubic-bezier(0.6, 0, 0.3, 1), margin 420ms cubic-bezier(0.6, 0, 0.3, 1);
  opacity: 0.28;
  filter: saturate(0.5) brightness(0.7);
}

.has-pick :deep(.spine.picked) {
  animation: none;
  opacity: 1;
  filter: none;
  transform: translateY(-14px);
  box-shadow: 0 0 0 1px rgb(147 196 86 / 0.5), 0 0 26px rgb(147 196 86 / 0.45),
  0 10px 18px rgb(0 0 0 / 0.5);
}

/* Pulled off the shelf: rises away first, then the neighbours close the gap. */
.has-pick :deep(.spine.taken) {
  transition: transform 520ms cubic-bezier(0.4, 0, 0.4, 1), opacity 400ms ease 160ms,
  box-shadow 300ms ease,
  width 400ms cubic-bezier(0.5, 0, 0.3, 1) 220ms, margin 400ms cubic-bezier(0.5, 0, 0.3, 1) 220ms;
  transform: translateY(-70px);
  opacity: 0;
  width: 0 !important;
  margin-left: -3px;
  box-shadow: none;
}

@media (prefers-reduced-motion: reduce) {
  :deep(.spine.gust) {
    animation: none;
  }
}
</style>
