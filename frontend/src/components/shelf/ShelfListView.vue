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
          :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
          :book-id="book.id"
          hoverable
          @resolve-cover="onResolveCover"
      />
      <div class="flex-1 min-w-0 flex flex-col justify-center">
        <h3 class="t-title text-[15px] md:text-base truncate group-hover:text-green-soft transition-colors duration-150">{{ book.title }}</h3>
        <p class="t-meta mt-0.5">{{ book.author }}</p>
        <p class="t-mono mt-1 hidden md:block">{{ dateLabel }} {{ formatDate(book) }}</p>
        <Stars v-if="book.rating" :rating="book.rating" :size="12" class="mt-0.5"/>
      </div>
      <Button variant="ghost" class="px-2! py-2! text-[13px]!" @click="$emit('removeBook', book.id)">
        <MinusIcon class="size-4"/>
      </Button>
    </li>
  </ul>
</template>

<script setup lang="ts">
import {MinusIcon} from '@heroicons/vue/24/outline';
import BookCover from '@/components/ui/BookCover.vue';
import Stars from '@/components/ui/Stars.vue';
import {bookCoverUrl} from '@/utils/coverUrl';
import {useBookCovers} from '@/composables/useBookCovers';
import moment from 'moment';
import type {ShelfBook} from '@/types/shelf';
import Button from "@/components/ui/Button.vue";

const props = withDefaults(defineProps<{
  books: ShelfBook[];
  coverWidth: number;
  dateLabel?: string;
  dateField?: 'added_at' | 'finished_at';
}>(), {
  dateLabel: 'Added',
  dateField: 'added_at',
});

defineEmits<{
  viewBook: [id: string];
  removeBook: [id: string];
}>();

const { resolvedCoverUrl, onResolveCover } = useBookCovers();

const formatDate = (book: ShelfBook) => {
  const val = (book as any)[props.dateField];
  if (!val) return '';
  return moment(val).format('MMM D, YYYY');
};
</script>
