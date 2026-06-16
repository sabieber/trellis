<template>
  <div class="min-h-screen">
    <div class="flex flex-col">
      <!-- Back -->
      <div class="px-4 pt-4 pb-2">
        <button
            @click="$router.back()"
            class="flex items-center gap-1 t-meta text-ink-dim hover:text-ink transition-colors duration-150"
        >
          <ChevronLeftIcon class="size-4"/>
          Back
        </button>
      </div>

      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <div v-else-if="book" class="px-4 pb-8">
        <!-- Book header -->
        <div class="flex gap-4 mb-6">
          <BookCover
              :title="book.volumeInfo.title || 'Untitled'"
              :author="book.volumeInfo.authors?.join(', ') || ''"
              :width="108"
              :cover-url="book.volumeInfo.imageLinks?.thumbnail"
          />
          <div class="flex flex-col justify-end min-w-0">
            <h1 class="t-display text-[21px]">{{ book.volumeInfo.title }}</h1>
            <p class="t-meta text-sm mt-1">{{ book.volumeInfo.authors?.join(', ') }}</p>
            <p class="t-meta mt-2">
              {{ book.volumeInfo.publishedDate?.slice(0, 4) }}
              <span v-if="book.volumeInfo.pageCount"> · {{ book.volumeInfo.pageCount }} pp</span>
              <span v-if="book.volumeInfo.categories?.[0]"> · {{ book.volumeInfo.categories[0] }}</span>
            </p>
          </div>
        </div>

        <!-- Tabs -->
        <div class="mb-5">
          <SegmentedControl v-model="activeTab" :options="tabs" class="w-full"/>
        </div>

        <!-- Info tab -->
        <div v-if="activeTab === 'Info'">
          <template v-if="book.volumeInfo.description">
            <h2 class="t-eyebrow mb-2">About</h2>
            <p class="text-ink-dim text-sm leading-relaxed mb-5" v-html="book.volumeInfo.description"></p>
          </template>
          <h2 class="t-eyebrow mb-1">Details</h2>
          <div class="flex flex-col">
            <div v-if="book.volumeInfo.categories?.[0]" class="flex justify-between py-3 border-b border-line-soft">
              <span class="t-meta">Genre</span>
              <span class="text-sm font-semibold text-green-soft">{{ book.volumeInfo.categories[0] }}</span>
            </div>
            <div v-if="book.volumeInfo.publishedDate" class="flex justify-between py-3 border-b border-line-soft">
              <span class="t-meta">Published</span>
              <span class="text-sm font-semibold text-ink">{{ book.volumeInfo.publishedDate.slice(0, 4) }}</span>
            </div>
            <div v-if="book.volumeInfo.pageCount" class="flex justify-between py-3 border-b border-line-soft">
              <span class="t-meta">Pages</span>
              <span class="t-mono text-ink!">{{ book.volumeInfo.pageCount }}</span>
            </div>
          </div>
        </div>

        <!-- Log tab -->
        <div v-else-if="activeTab === 'Log'">
          <div v-if="readings.length" class="flex flex-col mb-4">
            <div
                v-for="reading in readings"
                :key="reading.id"
                @click="viewReadingDetail(reading.id)"
                class="flex justify-between items-center py-3 border-b border-line-soft cursor-pointer group"
            >
              <span class="text-sm text-ink group-hover:text-green-soft transition-colors duration-150">{{
                  formatDate(reading.started_at)
                }}</span>
              <span class="t-meta">{{ reading.progress }} / {{ reading.total_pages }} pages</span>
            </div>
          </div>
          <p v-else class="t-meta text-center py-4">No reading sessions yet.</p>
          <Button variant="soft" block class="mt-2" @click="showStartReadingModal = true">
            <BookOpenIcon class="size-4"/>
            Start Reading
          </Button>
        </div>

        <!-- Shelves tab -->
        <div v-else-if="activeTab === 'Shelves'">
          <div v-if="loadingShelves" class="flex justify-center py-4">
            <span class="loading loading-spinner loading-md"></span>
          </div>
          <div v-else-if="shelves.length" class="flex flex-col">
            <div
                v-for="shelf in shelves"
                :key="shelf.id"
                @click="toggleShelf(shelf.id)"
                class="flex items-center justify-between py-3 border-b border-line-soft cursor-pointer"
            >
              <span class="text-sm" :class="isOnShelf(shelf.id) ? 'text-ink' : 'text-ink-dim'">{{ shelf.name }}</span>
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
        :initialPages="book?.volumeInfo?.pageCount || 0"
    />

    <!-- Toast -->
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
import {ChevronLeftIcon, BookOpenIcon, CheckIcon} from "@heroicons/vue/24/outline";
import {fetchBookDetails, searchBooks, resolveGoogleId} from '@/api/googleBooksApi';
import StartReadingModal from '@/components/StartReadingModal.vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import {apiFetch} from '@/api/client';
import moment from 'moment';

export default defineComponent({
  components: {ChevronLeftIcon, BookOpenIcon, CheckIcon, StartReadingModal, BookCover, Button, SegmentedControl},
  setup() {
    const route = useRoute();
    const router = useRouter();
    const book = ref<any>(null);
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
          return {googleBooksId: data.google_books_id as string | null, isbn13: data.isbn13 as string | null};
        }
      } catch (error) {
        console.error('Failed to fetch book info:', error);
      }
      return null;
    };

    const fetchBookDetailsWrapper = async (bookId: string) => {
      const info = await fetchBookInfo(bookId);
      let googleBooksId = info?.googleBooksId;
      if (!googleBooksId) {
        googleBooksId = await resolveGoogleId(bookId);
      }
      if (googleBooksId) {
        book.value = await fetchBookDetails(googleBooksId);
      } else if (info?.isbn13) {
        const results = await searchBooks(`isbn:${info.isbn13}`);
        if (results.length > 0) book.value = results[0];
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
            title: book.value.volumeInfo.title,
            author: book.value.volumeInfo.authors?.join(', '),
            isbn13: book.value.volumeInfo.industryIdentifiers?.find((id: any) => id.type === 'ISBN_13')?.identifier,
            isbn10: book.value.volumeInfo.industryIdentifiers?.find((id: any) => id.type === 'ISBN_10')?.identifier,
            google_books_id: book.value.id,
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

    const removeBookFromShelf = async (shelfId: string) => {
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
        isOnShelf(shelfId) ? removeBookFromShelf(shelfId) : addBookToShelf(shelfId);

    const viewReadingDetail = (readingId: string) => {
      router.push({name: 'reading-detail', params: {id: readingId}});
    };

    const startReadingSession = async (totalPages: number) => {
      try {
        const response = await apiFetch('/api/books/start-reading', {
          method: 'POST',
          body: JSON.stringify({book_id: route.params.id, total_pages: totalPages}),
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
      toastMessage,
      toastType,
      viewReadingDetail,
      startReadingSession,
      isOnShelf,
      toggleShelf,
      formatDate,
    };
  },
});
</script>
