<template>
  <div class="flex flex-wrap gap-3">
    <RouterLink
        v-for="book in books"
        :key="book.id"
        :to="{ name: 'book-detail', params: { id: book.id } }"
    >
      <BookCover
          :title="book.title"
          :author="book.author"
          :width="tileWidth"
          :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
          :book-id="book.id"
          :rating="book.rating"
          hoverable
          @resolve-cover="onResolveCover"
      />
    </RouterLink>
  </div>
</template>

<script setup lang="ts">
import BookCover from '@/components/ui/BookCover.vue';
import {bookCoverUrl} from '@/utils/coverUrl';
import {useBookCovers} from '@/composables/useBookCovers';
import type {ShelfBook} from '@/types/shelf';

defineProps<{
  books: ShelfBook[];
  tileWidth: number;
}>();

const { resolvedCoverUrl, onResolveCover } = useBookCovers();
</script>
