<template>
  <div class="flex flex-wrap gap-3">
    <BookCover
        v-for="book in books"
        :key="book.id"
        :title="book.title"
        :author="book.author"
        :width="tileWidth"
        :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
        :book-id="book.id"
        :rating="book.rating"
        hoverable
        class="cursor-pointer"
        @click="$emit('viewBook', book.id)"
        @resolve-cover="onResolveCover"
    />
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

defineEmits<{
  viewBook: [id: string];
}>();

const { resolvedCoverUrl, onResolveCover } = useBookCovers();
</script>
