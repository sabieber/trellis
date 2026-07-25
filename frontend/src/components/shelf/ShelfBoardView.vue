<template>
  <div class="flex flex-col">
    <div v-for="(row, ri) in shelfRows" :key="ri" class="shelf-row">
      <div class="shelf-books">
        <BookSpine
            v-for="book in row"
            :key="book.id"
            :title="book.title"
            :author="book.author"
            :page-count="book.page_count"
            :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
            :book-id="book.id"
            :height="spineHeight"
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
import {spineWidth} from '@/utils/bookColorway';
import {bookCoverUrl} from '@/utils/coverUrl';
import {useBookCovers} from '@/composables/useBookCovers';
import type {ShelfBook} from '@/types/shelf';

const {resolvedCoverUrl, onResolveCover} = useBookCovers();

const SHELF_GAP = 3;

const props = defineProps<{
  books: ShelfBook[];
  spineHeight: number;
  containerWidth: number;
}>();

defineEmits<{
  viewBook: [id: string];
}>();

const shelfRows = computed(() => {
  const w = props.containerWidth;
  if (w <= 0 || !props.books.length) return [props.books];
  const rows: ShelfBook[][] = [];
  let row: ShelfBook[] = [];
  let rowWidth = 0;
  for (const book of props.books) {
    const width = spineWidth(book.title, book.page_count);
    const needed = row.length === 0 ? width : width + SHELF_GAP;
    if (rowWidth + needed > w && row.length > 0) {
      rows.push(row);
      row = [book];
      rowWidth = width;
    } else {
      row.push(book);
      rowWidth += needed;
    }
  }
  if (row.length) rows.push(row);
  return rows;
});
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
</style>
