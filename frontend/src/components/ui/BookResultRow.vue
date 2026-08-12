<!-- One catalog hit as a list row, in the shape the search results use. The
     whole row is the link, so it is a plain `RouterLink` (see CLAUDE.md). Use it
     for books the user does not own — an owned book has its own detail page and
     belongs in `BookLayout`. -->
<template>
  <RouterLink
      :to="{ name: 'search-detail', params: { id: book.id } }"
      class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer group"
  >
    <BookCover
        :title="book.title || $t('common.untitled')"
        :author="book.authors?.join(', ') || ''"
        :width="46"
        :cover-url="book.cover_url"
        hoverable
    />
    <div class="min-w-0 flex flex-col justify-center">
      <h3 class="t-title text-[15px] truncate group-hover:text-green-soft transition-colors duration-150">
        {{ book.title }}
      </h3>
      <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
      <p class="t-meta mt-1">
        {{ book.published_year }}
        <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
      </p>
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import BookCover from '@/components/ui/BookCover.vue';
import type {BookSearchResult} from '@/types/book';

defineProps<{ book: BookSearchResult }>();
</script>
