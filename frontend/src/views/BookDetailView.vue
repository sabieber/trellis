<template>
  <div class="min-h-screen">
    <div class="flex flex-col">
      <div class="px-4 pt-4 pb-2">
        <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" @click="$router.back()">
          <ChevronLeftIcon class="size-4"/>
          Back
        </Button>
      </div>

      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <div v-else-if="book" class="px-4 pb-8">
        <div class="flex gap-4 mb-6">
          <BookCover
              :title="book.title || 'Untitled'"
              :author="book.authors?.join(', ') || ''"
              :width="108"
              :cover-url="book.cover_url"
          />
          <div class="flex flex-col justify-end min-w-0">
            <h1 class="t-display text-[21px]">{{ book.title }}</h1>
            <p class="t-meta text-sm mt-1">{{ book.authors?.join(', ') }}</p>
            <div class="mt-2">
              <Stars :rating="rating" :size="18" interactive @update="rateBook"/>
            </div>
            <p class="t-meta mt-2">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ book.page_count }} pp</span>
              <span v-if="book.category"> · {{ book.category }}</span>
            </p>
          </div>
        </div>

        <div class="mb-5">
          <SegmentedControl v-model="activeTab" :options="tabs" class="w-full"/>
        </div>

        <div v-if="activeTab === 'Info'">
          <template v-if="book.description">
            <h2 class="t-eyebrow mb-2">About</h2>
            <p class="text-ink-dim text-sm leading-relaxed mb-5" v-html="book.description"></p>
          </template>
          <h2 class="t-eyebrow mb-1">Details</h2>
          <div class="flex flex-col">
            <div v-if="book.category" class="flex justify-between py-3 border-b border-line-soft">
              <span class="t-meta">Genre</span>
              <span class="text-sm font-semibold text-green-soft">{{ book.category }}</span>
            </div>
            <div v-if="book.published_year" class="flex justify-between py-3 border-b border-line-soft">
              <span class="t-meta">Published</span>
              <span class="text-sm font-semibold text-ink">{{ book.published_year }}</span>
            </div>
            <div v-if="book.page_count" class="flex justify-between py-3 border-b border-line-soft">
              <span class="t-meta">Pages</span>
              <span class="t-mono text-ink!">{{ book.page_count }}</span>
            </div>
          </div>
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
                <span class="t-meta group-hover:text-green-soft transition-colors duration-150">{{ reading.progress }} / {{ reading.total_pages }} pages</span>
              </div>
              <button
                  @click.stop="confirmDeleteReading(reading.id)"
                  class="flex items-center justify-center size-7 rounded-full flex-none ml-2 text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
              >
                <TrashIcon class="size-4"/>
              </button>
            </div>
          </div>
          <p v-else class="t-meta text-center py-4">No reading sessions yet.</p>
          <Button variant="soft" block class="mt-2" @click="showStartReadingModal = true">
            <BookOpenIcon class="size-4"/>
            Start Reading
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
              <span class="text-sm group-hover:text-green-soft transition-colors duration-150" :class="isOnShelf(shelf.id) ? 'text-ink' : 'text-ink-dim'">{{ shelf.name }}</span>
              <div
                  class="size-7 rounded-full flex items-center justify-center border transition-colors duration-150"
                  :class="isOnShelf(shelf.id) ? 'bg-green/13 border-green/32' : 'bg-surface border-line'"
              >
                <CheckIcon v-if="isOnShelf(shelf.id)" class="size-4 text-green-soft"/>
              </div>
            </div>
          </div>
          <div v-else class="t-meta text-center py-4">No shelves found.</div>
        </div>
      </div>

      <div v-else class="t-meta text-center py-8 px-4">Book not found.</div>
    </div>

    <StartReadingModal
        v-if="showStartReadingModal"
        @close="showStartReadingModal = false"
        @submit="startReadingSession"
        :initialPages="book?.page_count || 0"
    />

    <ConfirmDialog
        v-if="pendingRemoveShelfId"
        title="Remove from Shelf"
        message="Are you sure you want to remove this book from the shelf?"
        confirmLabel="Remove"
        @confirm="removeBookFromShelf"
        @cancel="pendingRemoveShelfId = null"
    />

    <ConfirmDialog
        v-if="pendingDeleteReadingId"
        title="Delete Reading"
        message="Are you sure you want to delete this reading session and all its entries? This cannot be undone."
        confirmLabel="Delete"
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
import {defineComponent, ref, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {ChevronLeftIcon, BookOpenIcon, CheckIcon, TrashIcon} from "@heroicons/vue/24/outline";
import {fetchBookDetail, searchBooks, resolveGoogleId} from '@/api/bookApi';
import StartReadingModal from '@/components/StartReadingModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import Stars from '@/components/ui/Stars.vue';
import {apiFetch} from '@/api/client';
import moment from 'moment';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {ChevronLeftIcon, BookOpenIcon, CheckIcon, TrashIcon, StartReadingModal, ConfirmDialog, BookCover, Button, SegmentedControl, Stars},
  setup() {
    const route = useRoute();
    const router = useRouter();
    const book = ref<BookSearchResult | null>(null);
    const readings = ref<Array<{
      id: string;
      started_at: string;
      finished_at: string | null;
      progress: number;
      total_pages: number
    }>>([]);
    const loading = ref(true);
    const showStartReadingModal = ref(false);
    const activeTab = ref((route.query.tab as string) || 'Info');
    const tabs = [
      { value: 'Info', label: 'Info' },
      { value: 'Log', label: 'Log' },
      { value: 'Shelves', label: 'Shelves' },
    ];
    const shelves = ref<Array<{ id: string; name: string; description: string }>>([]);
    const shelfIds = ref<string[]>([]);
    const loadingShelves = ref(false);
    const toastMessage = ref('');
    const toastType = ref('');
    const rating = ref<number>(0);
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
          showToast('Book added to shelf successfully.', 'alert-success');
        } else {
          showToast('Failed to add book to shelf.', 'alert-error');
        }
      } catch {
        showToast('Failed to add book to shelf.', 'alert-error');
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
          showToast('Book removed from shelf successfully.', 'alert-success');
        } else {
          const data = await response.json();
          showToast(data.error || 'Failed to remove book from shelf.', 'alert-error');
        }
      } catch {
        showToast('Failed to remove book from shelf.', 'alert-error');
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
          showToast('Reading deleted.', 'alert-success');
        } else {
          const data = await response.json();
          showToast(data.error || 'Failed to delete reading.', 'alert-error');
        }
      } catch {
        showToast('Failed to delete reading.', 'alert-error');
      }
    };

    const viewReadingDetail = (readingId: string) => {
      router.push({name: 'reading-detail', params: {id: readingId}});
    };

    const startReadingSession = async (totalPages: number, startedAt: string) => {
      try {
        const response = await apiFetch('/api/books/start-reading', {
          method: 'POST',
          body: JSON.stringify({book_id: route.params.id, total_pages: totalPages, started_at: startedAt}),
        });
        if (response.ok) {
          await fetchBookDetailsWrapper(route.params.id as string);
          showStartReadingModal.value = false;
        } else {
          const errorData = await response.json();
          showToast(errorData.error, 'alert-error');
        }
      } catch {
        showToast('Failed to start reading session.', 'alert-error');
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
        showToast('Failed to update rating.', 'alert-error');
      }
    };

    const formatDate = (date: string) => moment(date).format('LL');

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
      startReadingSession,
      isOnShelf,
      toggleShelf,
      removeBookFromShelf,
      confirmDeleteReading,
      deleteReading,
      rateBook,
      formatDate,
    };
  },
});
</script>
