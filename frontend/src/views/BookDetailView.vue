<template>
  <div class="min-h-screen">
    <div class="flex flex-col">
      <div class="px-4 pt-4 pb-2">
        <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" @click="$router.back()">
          <ChevronLeftIcon class="size-4"/>
          {{ $t('common.back') }}
        </Button>
      </div>

      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <div v-else-if="book" class="px-4 pb-8">
        <div class="flex gap-4 mb-6">
          <BookCover
              :title="book.title || $t('common.untitled')"
              :author="book.authors?.join(', ') || ''"
              :width="108"
              :cover-url="book.cover_url"
          />
          <div class="flex flex-col justify-end min-w-0">
            <h1 class="t-display text-[21px]">{{ book.title }}</h1>
            <p class="t-meta text-sm mt-1">
              <!-- links on the enriched Google-Books author name; matches
                   sibling books only if their stored `books.author` string agrees. -->
              <template v-for="(a, i) in (book.authors || [])" :key="a">
                <span v-if="i > 0">, </span>
                <span
                    class="hover:text-green-soft hover:underline transition-colors duration-150 cursor-pointer"
                    @click="viewAuthor(a)"
                >{{ a }}</span>
              </template>
            </p>
            <div class="mt-2">
              <Stars :rating="rating" :size="18" interactive @update="rateBook"/>
            </div>
            <p class="t-meta mt-2">
              {{ book.published_year }}
              <span v-if="displayedPageCount"> · {{ $t('search.pagesAbbr', { count: displayedPageCount }) }}</span>
              <span v-if="book.category"> · {{ book.category }}</span>
            </p>
          </div>
        </div>

        <div class="mb-5">
          <SegmentedControl v-model="activeTab" :options="tabs" class="w-full"/>
        </div>

        <!-- Description fills the left column (2/3); details & external links sit
             in a rail (1/3) on the right. Stacks to one column below lg. The rail
             is a single flex column (one grid cell) so it stays packed at the top
             regardless of the description's length. -->
        <div v-if="activeTab === 'Info'" class="lg:grid lg:grid-cols-3 lg:gap-x-10 lg:items-start">
          <div v-if="book.description" class="min-w-0 lg:col-span-2 lg:col-start-1 lg:row-start-1">
            <h2 class="t-eyebrow mb-2">{{ $t('bookDetail.about') }}</h2>
            <div
                class="text-ink-dim text-sm leading-relaxed [&_p]:mb-3 [&_p:last-child]:mb-0 [&_a]:text-green-soft [&_a]:underline [&_a]:underline-offset-2 hover:[&_a]:text-green"
                v-html="book.description"
            ></div>
          </div>

          <aside class="mt-8 lg:mt-0 lg:col-start-3 lg:row-start-1 flex flex-col gap-5">
            <div>
              <h2 class="t-eyebrow mb-1">{{ $t('bookDetail.details') }}</h2>
              <div class="flex flex-col">
                <div v-if="book.category" class="flex justify-between py-3 border-b border-line-soft">
                  <span class="t-meta">{{ $t('bookDetail.genre') }}</span>
                  <span class="text-sm font-semibold text-green-soft">{{ book.category }}</span>
                </div>
                <div v-if="book.published_year" class="flex justify-between py-3 border-b border-line-soft">
                  <span class="t-meta">{{ $t('bookDetail.published') }}</span>
                  <span class="text-sm font-semibold text-ink">{{ book.published_year }}</span>
                </div>
                <div class="flex justify-between py-3 border-b border-line-soft">
                  <span class="t-meta">{{ $t('common.pages') }}</span>
                  <InlineEdit
                      class="t-mono text-ink!"
                      :value="displayedPageCount"
                      type="number"
                      :label="$t('bookDetail.editPageCount')"
                      :validate="isValidPageCount"
                      :save="savePageCount"
                  />
                </div>
              </div>
            </div>

            <div v-if="sourceUrl || goodreadsUrl || amazonUrl" class="flex flex-col gap-2">
              <Button v-if="sourceUrl" variant="ghost" block @click="openExternal(sourceUrl)">
                <ArrowTopRightOnSquareIcon class="size-4"/>
                {{ $t('bookDetail.viewOn', { source: book.source === 'google' ? 'Google Books' : 'Open Library' }) }}
              </Button>
              <Button v-if="goodreadsUrl" variant="ghost" block @click="openExternal(goodreadsUrl)">
                <ArrowTopRightOnSquareIcon class="size-4"/>
                {{ $t('bookDetail.viewOn', { source: 'Goodreads' }) }}
              </Button>
              <Button v-if="amazonUrl" variant="ghost" block @click="openExternal(amazonUrl)">
                <ArrowTopRightOnSquareIcon class="size-4"/>
                {{ $t('bookDetail.viewOn', { source: 'Amazon' }) }}
              </Button>
            </div>
          </aside>
        </div>

        <div v-else-if="activeTab === 'Log'">
          <div v-if="readings.length" class="flex flex-col mb-4">
            <div
                v-for="reading in readings"
                :key="reading.id"
                class="flex justify-between items-center py-3 border-b border-line-soft group"
            >
              <div
                  @click="viewReadingDetail(reading.id)"
                  class="flex justify-between items-center flex-1 min-w-0 cursor-pointer"
              >
                <span class="text-sm text-ink group-hover:text-green-soft transition-colors duration-150">{{
                    formatDate(reading.started_at)
                  }}</span>
                <span class="flex items-center gap-2">
                  <span class="badge badge-sm" :class="readingState(reading).badgeClass">{{ readingState(reading).label }}</span>
                  <span class="badge badge-sm">{{ reading.mode === 'percentage' ? $t('readingModal.modePercentage') : $t('readingModal.modePages') }}</span>
                  <span class="t-meta group-hover:text-green-soft transition-colors duration-150">{{ reading.mode === 'percentage' ? `${reading.progress}%` : $t('bookDetail.pagesProgress', { current: reading.progress, total: reading.total_pages }) }}</span>
                </span>
              </div>
              <button
                  @click.stop="confirmDeleteReading(reading.id)"
                  class="flex items-center justify-center size-7 rounded-full flex-none ml-2 text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
              >
                <TrashIcon class="size-4"/>
              </button>
            </div>
          </div>
          <p v-else class="t-meta text-center py-4">{{ $t('bookDetail.noReadings') }}</p>
          <Button variant="soft" block class="mt-2" @click="showStartReadingModal = true">
            <BookOpenIcon class="size-4"/>
            {{ $t('readingModal.title') }}
          </Button>
        </div>

        <div v-else-if="activeTab === 'Shelves'">
          <div v-if="loadingShelves" class="flex justify-center py-4">
            <span class="loading loading-spinner loading-md"></span>
          </div>
          <div v-else-if="shelves.length" class="flex flex-col">
            <div
                v-for="shelf in shelves"
                :key="shelf.id"
                @click="toggleShelf(shelf.id)"
                class="flex items-center justify-between py-3 border-b border-line-soft cursor-pointer group"
            >
              <span class="text-sm group-hover:text-green-soft transition-colors duration-150" :class="isOnShelf(shelf.id) ? 'text-ink' : 'text-ink-dim'">{{ shelf.name || shelf.code }}</span>
              <div
                  class="size-7 rounded-full flex items-center justify-center border transition-colors duration-150"
                  :class="isOnShelf(shelf.id) ? 'bg-green/13 border-green/32' : 'bg-surface border-line'"
              >
                <CheckIcon v-if="isOnShelf(shelf.id)" class="size-4 text-green-soft"/>
              </div>
            </div>
          </div>
          <div v-else class="t-meta text-center py-4">{{ $t('addToShelf.noShelves') }}</div>
        </div>
      </div>

      <div v-else class="t-meta text-center py-8 px-4">{{ $t('common.bookNotFound') }}</div>
    </div>

    <StartReadingModal
        v-if="showStartReadingModal"
        @close="showStartReadingModal = false"
        @submit="startReadingSession"
        :initialPages="displayedPageCount || 0"
    />

    <ConfirmDialog
        v-if="pendingRemoveShelfId"
        :title="$t('bookDetail.removeFromShelfTitle')"
        :message="$t('shelf.removeBookMessage')"
        :confirmLabel="$t('common.remove')"
        @confirm="removeBookFromShelf"
        @cancel="pendingRemoveShelfId = null"
    />

    <ConfirmDialog
        v-if="pendingDeleteReadingId"
        :title="$t('bookDetail.deleteReadingTitle')"
        :message="$t('bookDetail.deleteReadingMessage')"
        @confirm="deleteReading"
        @cancel="pendingDeleteReadingId = null"
    />

    <div v-if="toastMessage" class="toast toast-top toast-center pt-4 z-50">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {goToAuthor} from '@/utils/authorRoute';
import {useI18n} from 'vue-i18n';
import {ChevronLeftIcon, BookOpenIcon, CheckIcon, TrashIcon, ArrowTopRightOnSquareIcon} from "@heroicons/vue/24/outline";
import {fetchBookDetail, searchBooks, resolveGoogleId} from '@/api/bookApi';
import {apiErrorMessage} from '@/utils/apiError';
import StartReadingModal from '@/components/StartReadingModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import InlineEdit from '@/components/ui/InlineEdit.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import Stars from '@/components/ui/Stars.vue';
import {apiFetch} from '@/api/client';
import moment from 'moment';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {ChevronLeftIcon, BookOpenIcon, CheckIcon, TrashIcon, ArrowTopRightOnSquareIcon, StartReadingModal, ConfirmDialog, BookCover, Button, InlineEdit, SegmentedControl, Stars},
  setup() {
    const {t} = useI18n();
    const route = useRoute();
    const router = useRouter();
    const book = ref<BookSearchResult | null>(null);
    const readings = ref<Array<{
      id: string;
      started_at: string;
      finished_at: string | null;
      cancelled_at: string | null;
      progress: number;
      total_pages: number;
      mode: string;
    }>>([]);
    const loading = ref(true);
    const showStartReadingModal = ref(false);
    const activeTab = ref((route.query.tab as string) || 'Info');
    const tabs = computed(() => [
      { value: 'Info', label: t('bookDetail.tabInfo') },
      { value: 'Log', label: t('bookDetail.tabLog') },
      { value: 'Shelves', label: t('bookDetail.tabShelves') },
    ]);
    const shelves = ref<Array<{ id: string; code: string; name: string | null; description: string }>>([]);
    const shelfIds = ref<string[]>([]);
    const loadingShelves = ref(false);
    const toastMessage = ref('');
    const toastType = ref('');
    const rating = ref<number>(0);
    // User-provided page count, stored on the book row; takes precedence over
    // the page count reported by the external catalogs.
    const pageCountOverride = ref<number | null>(null);
    const pendingRemoveShelfId = ref<string | null>(null);
    const pendingDeleteReadingId = ref<string | null>(null);

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    const fetchBookInfo = async (bookId: string) => {
      try {
        const response = await apiFetch('/api/books/info', {
          method: 'POST',
          body: JSON.stringify({book_id: bookId}),
        });
        if (response.ok) {
          const data = await response.json();
          readings.value = data.readings;
          shelfIds.value = data.shelf_ids ?? [];
          rating.value = data.rating ?? 0;
          pageCountOverride.value = data.page_count ?? null;
          return {
            googleBooksId: data.google_books_id as string | null,
            openLibraryId: data.open_library_id as string | null,
            isbn13: data.isbn13 as string | null,
          };
        }
      } catch (error) {
        console.error('Failed to fetch book info:', error);
      }
      return null;
    };

    const fetchBookDetailsWrapper = async (bookId: string) => {
      const info = await fetchBookInfo(bookId);
      if (info?.googleBooksId) {
        book.value = await fetchBookDetail('google', info.googleBooksId);
      } else if (info?.openLibraryId) {
        book.value = await fetchBookDetail('openlibrary', info.openLibraryId);
      } else {
        let googleBooksId = await resolveGoogleId(bookId);
        if (googleBooksId) {
          book.value = await fetchBookDetail('google', googleBooksId);
        } else if (info?.isbn13) {
          const results = await searchBooks(`isbn:${info.isbn13}`);
          if (results.length > 0) book.value = results[0];
        }
      }
      loading.value = false;
    };

    const fetchShelves = async () => {
      loadingShelves.value = true;
      try {
        const response = await apiFetch('/api/shelves', {method: 'POST'});
        if (response.ok) {
          const data = await response.json();
          shelves.value = data.shelves;
        }
      } catch (error) {
        console.error('Failed to fetch shelves:', error);
      } finally {
        loadingShelves.value = false;
      }
    };

    const isOnShelf = (shelfId: string) => shelfIds.value.includes(shelfId);

    const addBookToShelf = async (shelfId: string) => {
      if (!book.value) return;
      try {
        const response = await apiFetch('/api/shelves/add-book', {
          method: 'POST',
          body: JSON.stringify({
            shelf_id: shelfId,
            title: book.value.title,
            author: book.value.authors?.join(', '),
            isbn13: book.value.isbn13,
            isbn10: book.value.isbn10,
            google_books_id: book.value.source === 'google' ? book.value.source_id : null,
            open_library_id: book.value.source === 'openlibrary' ? book.value.source_id : null,
            cover_url: book.value.cover_url,
          }),
        });
        if (response.ok) {
          if (!shelfIds.value.includes(shelfId)) shelfIds.value.push(shelfId);
          showToast(t('addToShelf.added'), 'alert-success');
        } else {
          showToast(t('addToShelf.addFailed'), 'alert-error');
        }
      } catch {
        showToast(t('addToShelf.addFailed'), 'alert-error');
      }
    };

    const confirmRemoveFromShelf = (shelfId: string) => {
      pendingRemoveShelfId.value = shelfId;
    };

    const removeBookFromShelf = async () => {
      const shelfId = pendingRemoveShelfId.value;
      if (!shelfId) return;
      pendingRemoveShelfId.value = null;
      try {
        const response = await apiFetch('/api/shelves/remove-book', {
          method: 'POST',
          body: JSON.stringify({book_id: route.params.id, shelf_id: shelfId}),
        });
        if (response.ok) {
          shelfIds.value = shelfIds.value.filter((id) => id !== shelfId);
          showToast(t('shelf.bookRemoved'), 'alert-success');
        } else {
          showToast(apiErrorMessage(response.status, t), 'alert-error');
        }
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
    };

    const toggleShelf = (shelfId: string) =>
        isOnShelf(shelfId) ? confirmRemoveFromShelf(shelfId) : addBookToShelf(shelfId);

    const confirmDeleteReading = (readingId: string) => {
      pendingDeleteReadingId.value = readingId;
    };

    const deleteReading = async () => {
      const readingId = pendingDeleteReadingId.value;
      if (!readingId) return;
      pendingDeleteReadingId.value = null;
      try {
        const response = await apiFetch('/api/readings/delete', {
          method: 'POST',
          body: JSON.stringify({reading_id: readingId}),
        });
        if (response.ok) {
          readings.value = readings.value.filter((r) => r.id !== readingId);
          showToast(t('bookDetail.readingDeleted'), 'alert-success');
        } else {
          showToast(apiErrorMessage(response.status, t), 'alert-error');
        }
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
    };

    const viewReadingDetail = (readingId: string) => {
      router.push({name: 'reading-detail', params: {id: readingId}});
    };

    const viewAuthor = (author: string) => goToAuthor(router, author);

    const startReadingSession = async (mode: string, totalPages: number, startedAt: string) => {
      try {
        const response = await apiFetch('/api/books/start-reading', {
          method: 'POST',
          body: JSON.stringify({book_id: route.params.id, total_pages: totalPages, started_at: startedAt, mode}),
        });
        if (response.ok) {
          await fetchBookDetailsWrapper(route.params.id as string);
          showStartReadingModal.value = false;
        } else {
          showToast(apiErrorMessage(response.status, t), 'alert-error');
        }
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
    };

    const rateBook = async (val: number | null) => {
      const bookId = route.params.id as string;
      try {
        const response = await apiFetch('/api/books/rate', {
          method: 'POST',
          body: JSON.stringify({book_id: bookId, rating: val}),
        });
        if (response.ok) {
          rating.value = val ?? 0;
        }
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
    };

    const displayedPageCount = computed(() =>
        pageCountOverride.value ?? book.value?.page_count ?? null);

    const isValidPageCount = (value: string) => /^\d+$/.test(value) && parseInt(value, 10) > 0;

    const savePageCount = async (value: string): Promise<boolean> => {
      const pages = parseInt(value, 10);
      try {
        const response = await apiFetch('/api/books/set-page-count', {
          method: 'POST',
          body: JSON.stringify({book_id: route.params.id, page_count: pages}),
        });
        if (response.ok) {
          pageCountOverride.value = pages;
          return true;
        }
        showToast(apiErrorMessage(response.status, t), 'alert-error');
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
      return false;
    };

    const formatDate = (date: string) => moment(date).format('LL');

    const readingState = (reading: { finished_at: string | null; cancelled_at: string | null }) => {
      if (reading.cancelled_at) return {label: t('bookDetail.stateAbandoned'), badgeClass: 'badge-neutral'};
      if (reading.finished_at) return {label: t('bookDetail.stateFinished'), badgeClass: 'badge-success'};
      return {label: t('bookDetail.stateReading'), badgeClass: 'badge-warning'};
    };

    const sourceUrl = computed(() => {
      if (!book.value) return null;
      if (book.value.source === 'google') {
        return `https://books.google.com/books?id=${encodeURIComponent(book.value.source_id)}`;
      }
      const key = book.value.source_id.startsWith('/')
          ? book.value.source_id
          : `/${book.value.source_id}`;
      return `https://openlibrary.org${key}`;
    });

    // Shared lookup query for retailers/review sites: prefer the ISBN, which
    // lands directly on the book page on exact-match search; otherwise fall
    // back to title + first author.
    const externalSearchQuery = computed(() => {
      if (!book.value) return null;
      const isbn = book.value.isbn13 || book.value.isbn10;
      if (isbn) return encodeURIComponent(isbn);
      const author = book.value.authors?.[0];
      const fallback = [book.value.title, author].filter(Boolean).join(' ');
      return fallback ? encodeURIComponent(fallback) : null;
    });

    const goodreadsUrl = computed(() =>
        externalSearchQuery.value ? `https://www.goodreads.com/search?q=${externalSearchQuery.value}` : null);

    const amazonUrl = computed(() =>
        externalSearchQuery.value ? `https://www.amazon.com/s?k=${externalSearchQuery.value}` : null);

    const openExternal = (url: string | null) => {
      if (url) window.open(url, '_blank', 'noopener');
    };

    onMounted(() => {
      const bookId = route.params.id as string;
      fetchBookDetailsWrapper(bookId);
      fetchShelves();
    });

    return {
      book,
      readings,
      loading,
      showStartReadingModal,
      activeTab,
      tabs,
      shelves,
      loadingShelves,
      rating,
      pendingRemoveShelfId,
      pendingDeleteReadingId,
      toastMessage,
      toastType,
      viewReadingDetail,
      viewAuthor,
      startReadingSession,
      isOnShelf,
      toggleShelf,
      removeBookFromShelf,
      confirmDeleteReading,
      deleteReading,
      rateBook,
      displayedPageCount,
      isValidPageCount,
      savePageCount,
      formatDate,
      readingState,
      sourceUrl,
      goodreadsUrl,
      amazonUrl,
      openExternal,
    };
  },
});
</script>
