<template>
  <div class="flex flex-wrap gap-3">
    <BookCover
        v-for="book in books"
        :key="book.id"
        :title="book.title"
        :author="book.author"
        :width="tileWidth"
        :cover-url="bookCoverUrl(book)"
        :rating="book.rating"
        hoverable
        class="cursor-pointer"
        @click="$emit('viewBook', book.id)"
    />
  </div>
</template>

<script setup lang="ts">
import BookCover from '@/components/ui/BookCover.vue';
import {bookCoverUrl} from '@/utils/coverUrl';
import type {ShelfBook} from '@/types/shelf';

defineProps<{
  books: ShelfBook[];
  tileWidth: number;
}>();

defineEmits<{
  viewBook: [id: string];
}>();
</script>
