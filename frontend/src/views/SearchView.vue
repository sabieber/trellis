<template>
  <div class="min-h-screen">
    <div class="px-4 pt-5">
      <h1 class="t-display text-2xl mb-4">Search</h1>

      <div
          class="flex items-center gap-2.5 bg-surface border border-line rounded-sm px-3.5 mb-5 transition-colors duration-150 focus-within:border-green/32"
      >
        <MagnifyingGlassIcon class="size-5 text-muted flex-none"/>
        <input
            type="text"
            v-model="query"
            class="w-full bg-transparent py-3 text-sm text-ink placeholder:text-muted focus:outline-none"
            placeholder="Title, author, or ISBN"
            @keyup.enter="searchBooks"
        />
        <button
            class="text-muted hover:text-ink transition-colors flex-none"
            title="Scan barcode"
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
              :title="book.title || 'Untitled'"
              :author="book.authors?.join(', ') || ''"
              :width="46"
              :cover-url="book.cover_url"
              hoverable
          />
          <div class="min-w-0 flex flex-col justify-center">
            <h3 class="t-title text-[15px] truncate group-hover:text-green-soft transition-colors duration-150">{{ book.title }}</h3>
            <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
            <p class="t-meta mt-1">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ book.page_count }} pp</span>
              <span v-if="book.category"> · {{ book.category }}</span>
            </p>
          </div>
        </div>
      </div>

      <div v-else-if="hasSearched" class="flex flex-col items-center text-center pt-12">
        <h3 class="t-title text-[17px]">Nothing's grown here yet</h3>
        <p class="t-meta mt-1.5 max-w-58">Search by title, author, or ISBN to find books.</p>
      </div>

      <div v-else-if="trendingBooks.length">
        <h2 class="t-title text-sm text-muted uppercase tracking-wide mb-3">Trending now</h2>
        <div
            v-for="book in trendingBooks"
            :key="book.id"
            class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer group"
            @click="viewBookDetail(book)"
        >
          <BookCover
              :title="book.title || 'Untitled'"
              :author="book.authors?.join(', ') || ''"
              :width="46"
              :cover-url="book.cover_url"
              hoverable
          />
          <div class="min-w-0 flex flex-col justify-center">
            <h3 class="t-title text-[15px] truncate group-hover:text-green-soft transition-colors duration-150">{{ book.title }}</h3>
            <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
            <p class="t-meta mt-1">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ book.page_count }} pp</span>
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
import {useRouter, useRoute} from 'vue-router';
import {MagnifyingGlassIcon, QrCodeIcon} from '@heroicons/vue/24/outline';
import BookCover from '@/components/ui/BookCover.vue';
import BarcodeScanner from '@/components/BarcodeScanner.vue';
import {searchBooks, fetchTrendingBooks} from '@/api/bookApi';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {MagnifyingGlassIcon, QrCodeIcon, BookCover, BarcodeScanner},
  setup() {
    const query = ref('');
    const books = ref<BookSearchResult[]>([]);
    const trendingBooks = ref<BookSearchResult[]>([]);
    const loading = ref(false);
    const hasSearched = ref(false);
    const showScanner = ref(false);
    const router = useRouter();
    const route = useRoute();

    const searchBooksWrapper = async () => {
      if (!query.value.trim()) return;
      loading.value = true;
      hasSearched.value = true;
      books.value = await searchBooks(query.value);
      loading.value = false;
      router.replace({query: {q: query.value}});
    };

    const viewBookDetail = (book: BookSearchResult) => {
      router.push({name: 'search-detail', params: {id: book.id}});
    };

    const onBarcodeDetected = (code: string) => {
      query.value = code;
      showScanner.value = false;
      searchBooksWrapper();
    };

    onMounted(async () => {
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
      onBarcodeDetected,
    };
  },
});
</script>
