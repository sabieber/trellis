<template>
  <ul class="flex flex-col">
    <li
        v-for="book in books"
        :key="book.id"
        class="flex items-center gap-4 py-3 md:py-4 border-b border-line-soft cursor-pointer group"
        @click="$emit('viewBook', book.id)"
    >
      <BookCover
          :title="book.title"
          :author="book.author"
          :width="coverWidth"
          :cover-url="bookCoverUrl(book)"
      />
      <div class="flex-1 min-w-0 flex flex-col justify-center">
        <h3 class="t-title text-[15px] md:text-base truncate">{{ book.title }}</h3>
        <p class="t-meta mt-0.5">{{ book.author }}</p>
        <p class="t-mono mt-1 hidden md:block">Added {{ formatAddedAt(book.added_at) }}</p>
      </div>
      <button
          @click.stop="$emit('removeBook', book.id)"
          class="self-center flex-none flex items-center justify-center size-8 md:size-9 rounded-full text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
      >
        <MinusIcon class="size-4 md:size-5"/>
      </button>
    </li>
  </ul>
</template>

<script setup lang="ts">
import {MinusIcon} from '@heroicons/vue/24/outline';
import BookCover from '@/components/ui/BookCover.vue';
import {bookCoverUrl} from '@/utils/coverUrl';
import moment from 'moment';
import type {ShelfBook} from '@/types/shelf';

defineProps<{
  books: ShelfBook[];
  coverWidth: number;
}>();

defineEmits<{
  viewBook: [id: string];
  removeBook: [id: string];
}>();

const formatAddedAt = (dateStr: string) => moment(dateStr).format('MMM D, YYYY');
</script>
