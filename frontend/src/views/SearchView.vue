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

      <div v-else-if="hasSearched && !books.length" class="flex flex-col items-center text-center pt-12">
        <h3 class="t-title text-[17px]">{{ $t('search.emptyTitle') }}</h3>
        <p class="t-meta mt-1.5 max-w-58">{{ $t('search.emptyHint') }}</p>
      </div>

      <!-- Results and the trending list are the same rows; only the heading
           tells them apart. -->
      <div v-else-if="displayedBooks.length">
        <div class="flex items-center gap-2 mb-3">
          <h2 v-if="!books.length" class="t-title text-sm text-muted uppercase tracking-wide">{{ $t('search.trending') }}</h2>
          <LayoutModeSelect v-model="layoutMode" class="ml-auto"/>
        </div>

        <!-- Only the row shows where a hit comes from and what editions it has.
             The other modes draw covers alone, so they show neither. -->
        <BookLayout v-if="layoutMode !== 'list'" :books="layoutBooks" :mode="layoutMode"/>

        <template v-for="book in (layoutMode === 'list' ? displayedBooks : [])" :key="book.id">
          <!-- Search adds two things to the shared row: where the hit comes
               from, and the toggle for the editions of a work. -->
          <BookResultRow :book="book">
            <template #badge>
              <span
                  class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border"
                  :class="book.source === 'library' ? 'border-green/40 text-green-soft' : 'border-line text-muted'"
              >{{ sourceLabel(book.source) }}</span>
            </template>
            <template #action>
              <button
                  v-if="isWork(book)"
                  type="button"
                  class="relative z-1 flex items-center justify-center size-8 rounded-full flex-none cursor-pointer transition-colors duration-150"
                  :class="expanded === book.id ? 'text-green-soft bg-surface-2' : 'text-muted hover:text-ink hover:bg-surface-2'"
                  :title="$t('search.editions')"
                  :aria-label="$t('search.editions')"
                  @click="toggleEditions(book)"
              >
                <ChevronDownIcon class="size-4 transition-transform duration-150" :class="expanded === book.id ? 'rotate-180' : ''"/>
              </button>
            </template>
          </BookResultRow>

          <div v-if="expanded === book.id" class="flex flex-col bg-surface/60">
            <div v-if="loadingEditions" class="flex justify-center py-4">
              <span class="loading loading-spinner loading-sm"></span>
            </div>
            <p v-else-if="!editions.length" class="t-meta py-3 pl-6">{{ $t('search.noEditions') }}</p>
            <RouterLink
                v-for="edition in editions"
                :key="edition.id"
                :to="{ name: 'search-detail', params: { id: edition.id } }"
                class="flex flex-col pl-6 py-2 border-b border-line-soft cursor-pointer group/edition"
            >
              <div class="flex items-center gap-2 min-w-0">
                <span
                    v-if="edition.language"
                    class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border border-green/40 text-green-soft"
                >{{ edition.language }}</span>
                <span class="text-sm text-ink truncate group-hover/edition:text-green-soft transition-colors duration-150">{{ edition.title }}</span>
              </div>
              <p class="t-meta mt-0.5 truncate">{{ editionMeta(edition) }}</p>
            </RouterLink>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, ref, onMounted} from 'vue';
import {useI18n} from 'vue-i18n';
import {useRouter, useRoute} from 'vue-router';
import {SearchIcon, QrCodeIcon, ChevronDownIcon} from '@lucide/vue';
import BookResultRow from '@/components/ui/BookResultRow.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import BarcodeScanner from '@/components/BarcodeScanner.vue';
import {searchBooks, fetchTrendingBooks} from '@/api/bookApi';
import {useEditions} from '@/composables/useEditions';
import {useLayoutMode} from '@/composables/useLayoutMode';
import {asShelfBook} from '@/utils/catalogBook';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {
    SearchIcon, QrCodeIcon, ChevronDownIcon, BookResultRow, BookLayout, LayoutModeSelect, BarcodeScanner,
  },
  setup() {
    // Its own key: search results are a different set of books from the shelf,
    // and the list is the only mode that shows the source and the editions.
    const layoutMode = useLayoutMode('search-layout-mode');
    const query = ref('');
    const books = ref<BookSearchResult[]>([]);
    const trendingBooks = ref<BookSearchResult[]>([]);
    const loading = ref(false);
    const hasSearched = ref(false);
    const showScanner = ref(false);
    const router = useRouter();
    const route = useRoute();
    const {t} = useI18n();
    const {expanded, editions, loadingEditions, isWork, toggleEditions, collapse, editionMeta} =
        useEditions();

    const displayedBooks = computed(() => (books.value.length ? books.value : trendingBooks.value));

    // Computed, not mapped in the template: the pile measures itself whenever
    // the list changes, and a fresh array on every keystroke is a fresh list.
    const layoutBooks = computed(() => displayedBooks.value.map(asShelfBook));

    const searchBooksWrapper = async () => {
      if (!query.value.trim()) return;
      loading.value = true;
      hasSearched.value = true;
      collapse();
      books.value = await searchBooks(query.value);
      loading.value = false;
      router.replace({query: {q: query.value}});
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
      displayedBooks,
      loading,
      hasSearched,
      showScanner,
      searchBooks: searchBooksWrapper,
      sourceLabel,
      onBarcodeDetected,
      layoutMode,
      layoutBooks,
      expanded,
      editions,
      loadingEditions,
      isWork,
      toggleEditions,
      editionMeta,
    };
  },
});
</script>
