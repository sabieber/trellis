<template>
  <div class="min-h-screen">
    <div class="px-4 pt-5">
      <h1 class="t-display text-2xl mb-4">{{ $t('nav.search') }}</h1>

      <div
          class="flex items-center gap-2.5 bg-surface border border-line rounded-sm px-3.5 mb-5 transition-colors duration-150 focus-within:border-green/32"
      >
        <SearchIcon class="size-5 text-muted flex-none"/>
        <!-- The id lets the bottom dock's search button focus this input when
             it is pressed while already on /search. -->
        <input
            id="search-input"
            type="text"
            v-model="query"
            class="w-full bg-transparent py-3 text-sm text-ink placeholder:text-muted focus:outline-none"
            :placeholder="$t('search.placeholder')"
            @keyup.enter="searchBooks"
        />
        <button
            class="text-muted hover:text-ink transition-colors flex-none"
            :title="$t('search.scanBarcode')"
            @click="showScanner = true"
        >
          <QrCodeIcon class="size-5"/>
        </button>
      </div>

      <BarcodeScanner
          v-if="showScanner"
          @detected="onBarcodeDetected"
          @close="showScanner = false"
      />

      <div v-if="loading" class="flex justify-center py-8">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <div v-else-if="books.length">
        <div
            v-for="book in books"
            :key="book.id"
            class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer group"
            @click="viewBookDetail(book)"
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
              <span
                  class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border"
                  :class="book.source === 'library' ? 'border-green/40 text-green-soft' : 'border-line text-muted'"
              >{{ sourceLabel(book.source) }}</span>
            </div>
            <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
            <p class="t-meta mt-1">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
              <span v-if="book.category"> · {{ book.category }}</span>
            </p>
          </div>
        </div>
      </div>

      <div v-else-if="hasSearched" class="flex flex-col items-center text-center pt-12">
        <h3 class="t-title text-[17px]">{{ $t('search.emptyTitle') }}</h3>
        <p class="t-meta mt-1.5 max-w-58">{{ $t('search.emptyHint') }}</p>
      </div>

      <div v-else-if="trendingBooks.length">
        <h2 class="t-title text-sm text-muted uppercase tracking-wide mb-3">{{ $t('search.trending') }}</h2>
        <div
            v-for="book in trendingBooks"
            :key="book.id"
            class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer group"
            @click="viewBookDetail(book)"
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
              <span
                  class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border"
                  :class="book.source === 'library' ? 'border-green/40 text-green-soft' : 'border-line text-muted'"
              >{{ sourceLabel(book.source) }}</span>
            </div>
            <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
            <p class="t-meta mt-1">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
              <span v-if="book.category"> · {{ book.category }}</span>
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, onMounted} from 'vue';
import {useI18n} from 'vue-i18n';
import {useRouter, useRoute} from 'vue-router';
import {SearchIcon, QrCodeIcon} from '@lucide/vue';
import BookCover from '@/components/ui/BookCover.vue';
import BarcodeScanner from '@/components/BarcodeScanner.vue';
import {searchBooks, fetchTrendingBooks} from '@/api/bookApi';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {SearchIcon, QrCodeIcon, BookCover, BarcodeScanner},
  setup() {
    const query = ref('');
    const books = ref<BookSearchResult[]>([]);
    const trendingBooks = ref<BookSearchResult[]>([]);
    const loading = ref(false);
    const hasSearched = ref(false);
    const showScanner = ref(false);
    const router = useRouter();
    const route = useRoute();
    const {t} = useI18n();

    const searchBooksWrapper = async () => {
      if (!query.value.trim()) return;
      loading.value = true;
      hasSearched.value = true;
      books.value = await searchBooks(query.value);
      loading.value = false;
      router.replace({query: {q: query.value}});
    };

    const viewBookDetail = (book: BookSearchResult) => {
      // Owned books already have a real row — go to their detail page, not the
      // external-lookup search-detail view (which would 404 on a UUID).
      if (book.source === 'library') {
        router.push({name: 'book-detail', params: {id: book.id}});
      } else {
        router.push({name: 'search-detail', params: {id: book.id}});
      }
    };

    const sourceLabel = (source: BookSearchResult['source']) =>
        t(`search.source.${source}`);

    const onBarcodeDetected = (code: string) => {
      query.value = code;
      showScanner.value = false;
      searchBooksWrapper();
    };

    onMounted(async () => {
      // Desktop only (Tailwind's `sm`): on mobile this would pop the keyboard
      // open on every visit and hide the trending list.
      if (window.matchMedia('(min-width: 640px)').matches) {
        document.getElementById('search-input')?.focus();
      }

      const savedQuery = route.query.q as string;
      if (savedQuery) {
        query.value = savedQuery;
        await searchBooksWrapper();
      } else {
        trendingBooks.value = await fetchTrendingBooks();
      }
    });

    return {
      query,
      books,
      trendingBooks,
      loading,
      hasSearched,
      showScanner,
      searchBooks: searchBooksWrapper,
      viewBookDetail,
      sourceLabel,
      onBarcodeDetected,
    };
  },
});
</script>
