<!-- One book as a list row, in the same shape as the shelf list (`ShelfListView`)
     so a screen that shows both lists reads as one list. The row carries its own
     author link, so the anchor sits on the title and its ::after covers the row
     (see CLAUDE.md).

     It takes a catalog hit (`BookSearchResult`) and routes by `source`: a book
     the user owns goes to its own detail page, a catalog hit to the
     external-lookup view. -->
<template>
  <div class="relative flex items-center gap-4 py-3 md:py-4 border-b border-line-soft cursor-pointer group">
    <BookCover
        :title="book.title || $t('common.untitled')"
        :author="book.authors?.join(', ') || ''"
        :width="isWide ? 72 : 56"
        :cover-url="book.cover_url"
        hoverable
    />
    <div class="flex-1 min-w-0 flex flex-col justify-center">
      <div class="flex items-center gap-2 min-w-0">
        <h3 class="t-title text-[15px] md:text-base truncate group-hover:text-green-soft transition-colors duration-150">
          <RouterLink class="stretched-link" :to="bookRoute(book)">{{ book.title }}</RouterLink>
        </h3>
        <slot name="badge"/>
      </div>
      <p class="t-meta mt-0.5 truncate">
        <!-- `relative z-1` keeps the author link above the stretched link. -->
        <template v-for="(author, i) in (book.authors || [])" :key="author">
          <span v-if="i > 0">, </span>
          <RouterLink
              v-if="isLinkableAuthor(author)"
              class="relative z-1 hover:text-green-soft hover:underline transition-colors duration-150"
              :to="authorRoute(author)"
          >{{ author }}</RouterLink>
          <span v-else>{{ author }}</span>
        </template>
      </p>
      <p class="t-meta mt-1">
        {{ book.published_year }}
        <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
        <span v-if="book.category"> · {{ book.category }}</span>
      </p>
    </div>
    <slot name="action"/>
  </div>
</template>

<script setup lang="ts">
import BookCover from '@/components/ui/BookCover.vue';
import {authorRoute, isLinkableAuthor} from '@/utils/authorRoute';
import {bookRoute} from '@/utils/bookRoute';
import {useIsWide} from '@/composables/useIsWide';
import type {BookSearchResult} from '@/types/book';

defineProps<{ book: BookSearchResult }>();

const isWide = useIsWide();
</script>
