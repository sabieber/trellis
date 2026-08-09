<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4 max-w-xl">
      <div>
        <h3 class="t-title text-lg">{{ $t('linkCatalog.title') }}</h3>
        <p class="t-meta mt-1">{{ $t('linkCatalog.hint') }}</p>
      </div>

      <!-- A form, not @keydown.enter: Android soft keyboards report keyCode 229
           while composing, so the key never matches there. -->
      <form @submit.prevent="search" class="flex gap-2">
        <div
            class="flex items-center gap-2.5 flex-1 bg-surface border border-line rounded-sm px-3.5 transition-colors duration-150 focus-within:border-green/32"
        >
          <SearchIcon class="size-5 text-muted flex-none"/>
          <input
              ref="input"
              type="text"
              v-model="query"
              class="w-full bg-transparent py-3 text-sm text-ink placeholder:text-muted focus:outline-none"
              :placeholder="$t('search.placeholder')"
          />
        </div>
        <Button type="submit" :disabled="loading">{{ $t('nav.search') }}</Button>
      </form>

      <div v-if="loading" class="flex justify-center py-8">
        <span class="loading loading-spinner loading-md"></span>
      </div>

      <div v-else-if="books.length" class="flex flex-col max-h-96 overflow-y-auto">
        <button
            v-for="book in books"
            :key="`${book.source}:${book.source_id}`"
            type="button"
            class="flex gap-3 py-2.5 text-left border-b border-line-soft cursor-pointer group"
            :disabled="submitting"
            @click="$emit('select', book)"
        >
          <BookCover
              :title="book.title || $t('common.untitled')"
              :author="book.authors?.join(', ') || ''"
              :width="46"
              :cover-url="book.cover_url"
              hoverable
          />
          <div class="min-w-0 flex flex-col justify-center">
            <div class="flex items-center gap-2 min-w-0">
              <h3 class="t-title text-[15px] truncate group-hover:text-green-soft transition-colors duration-150">{{ book.title }}</h3>
              <span class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border border-line text-muted">{{ $t(`search.source.${book.source}`) }}</span>
            </div>
            <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
            <p class="t-meta mt-1">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
              <span v-if="book.category"> · {{ book.category }}</span>
            </p>
          </div>
        </button>
      </div>

      <p v-else-if="hasSearched" class="t-meta text-center py-6">{{ $t('search.emptyTitle') }}</p>

      <div class="modal-action mt-0">
        <Button variant="ghost" block @click="$emit('close')">{{ $t('common.cancel') }}</Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import {defineComponent, onMounted, ref} from 'vue';
import {SearchIcon} from '@lucide/vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import {searchBooks} from '@/api/bookApi';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {SearchIcon, BookCover, Button},
  props: {
    initialQuery: {
      type: String,
      required: true,
    },
    // Set while the parent writes the pick, so a second click cannot fire.
    submitting: {
      type: Boolean,
      default: false,
    },
  },
  emits: ['close', 'select'],
  setup(props) {
    const query = ref(props.initialQuery);
    const books = ref<BookSearchResult[]>([]);
    const loading = ref(false);
    const hasSearched = ref(false);

    const search = async () => {
      if (!query.value.trim()) return;
      loading.value = true;
      hasSearched.value = true;
      // The user's own rows are what we are linking *from* — only catalog hits
      // can be linked to.
      books.value = (await searchBooks(query.value)).filter((b) => b.source !== 'library');
      loading.value = false;
    };

    onMounted(search);

    return {query, books, loading, hasSearched, search};
  },
});
</script>
