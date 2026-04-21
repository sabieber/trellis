<template>
  <div class="min-h-screen bg-gray-900">
    <div class="flex flex-col">
      <!-- Back -->
      <div class="px-4 pt-4 pb-2">
        <button @click="$router.back()" class="flex items-center gap-0.5 text-primary text-sm font-medium">
          <ChevronLeftIcon class="size-5" />
          Back
        </button>
      </div>

      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg text-primary"></span>
      </div>

      <div v-else-if="book" class="px-4 pb-8">
        <!-- Book header -->
        <div class="flex gap-4 mb-6">
          <img
            :src="book.volumeInfo.imageLinks?.thumbnail"
            alt="Book cover"
            class="w-24 h-32 object-cover rounded flex-shrink-0 bg-gray-700"
          />
          <div class="flex flex-col justify-center min-w-0">
            <h1 class="text-white font-bold text-lg leading-tight">{{ book.volumeInfo.title }}</h1>
            <p class="text-gray-400 text-sm mt-1">by {{ book.volumeInfo.authors?.join(', ') }}</p>
            <p class="text-gray-500 text-xs mt-1">
              {{ book.volumeInfo.publishedDate?.slice(0, 4) }}
              <span v-if="book.volumeInfo.pageCount"> · {{ book.volumeInfo.pageCount }} pages</span>
              <span v-if="book.volumeInfo.categories?.[0]"> · {{ book.volumeInfo.categories[0] }}</span>
            </p>
          </div>
        </div>

        <!-- Tabs -->
        <div class="flex border-b border-gray-800 mb-4">
          <button
            v-for="tab in tabs"
            :key="tab"
            @click="activeTab = tab"
            class="px-4 py-2 text-sm font-medium transition-colors"
            :class="activeTab === tab ? 'text-primary border-b-2 border-primary -mb-px' : 'text-gray-400'"
          >
            {{ tab }}
          </button>
        </div>

        <!-- Info tab -->
        <div v-if="activeTab === 'Info'">
          <p
            v-if="book.volumeInfo.description"
            class="text-gray-400 text-sm leading-relaxed mb-4"
            v-html="book.volumeInfo.description"
          ></p>
          <div class="flex flex-col divide-y divide-gray-800">
            <div v-if="book.volumeInfo.categories?.[0]" class="flex justify-between py-3">
              <span class="text-gray-400 text-sm">Genre</span>
              <span class="text-white text-sm font-semibold">{{ book.volumeInfo.categories[0] }}</span>
            </div>
            <div v-if="book.volumeInfo.publishedDate" class="flex justify-between py-3">
              <span class="text-gray-400 text-sm">Year</span>
              <span class="text-white text-sm font-semibold">{{ book.volumeInfo.publishedDate.slice(0, 4) }}</span>
            </div>
            <div v-if="book.volumeInfo.pageCount" class="flex justify-between py-3">
              <span class="text-gray-400 text-sm">Pages</span>
              <span class="text-white text-sm font-semibold">{{ book.volumeInfo.pageCount }}</span>
            </div>
          </div>
        </div>

        <!-- Log tab -->
        <div v-else-if="activeTab === 'Log'">
          <div v-if="readings.length" class="flex flex-col divide-y divide-gray-800 mb-4">
            <div
              v-for="reading in readings"
              :key="reading.id"
              @click="viewReadingDetail(reading.id)"
              class="flex justify-between items-center py-3 cursor-pointer"
            >
              <span class="text-white text-sm">{{ formatDate(reading.started_at) }}</span>
              <span class="text-gray-400 text-sm">{{ reading.progress }} / {{ reading.total_pages }} pages</span>
            </div>
          </div>
          <p v-else class="text-gray-500 text-sm text-center py-4">No reading sessions yet.</p>
          <button @click="showStartReadingModal = true" class="btn btn-outline btn-primary w-full mt-2">
            <BookOpenIcon class="size-5" />
            Start Reading
          </button>
        </div>

        <!-- Shelves tab -->
        <div v-else-if="activeTab === 'Shelves'">
          <div v-if="loadingShelves" class="flex justify-center py-4">
            <span class="loading loading-spinner loading-md"></span>
          </div>
          <div v-else-if="shelves.length" class="flex flex-col divide-y divide-gray-800">
            <div
              v-for="shelf in shelves"
              :key="shelf.id"
              @click="addBookToShelf(shelf.id)"
              class="flex items-center justify-between py-3 cursor-pointer"
            >
              <span class="text-white">{{ shelf.name }}</span>
              <div class="size-7 rounded-full bg-gray-700"></div>
            </div>
          </div>
          <div v-else class="text-gray-500 text-sm text-center py-4">No shelves found.</div>
        </div>
      </div>

      <div v-else class="text-gray-500 text-center py-8 px-4">Book not found.</div>
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
import { defineComponent, ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ChevronLeftIcon, BookOpenIcon } from "@heroicons/vue/24/solid";
import { fetchBookDetails, searchBooks } from '@/api/googleBooksApi';
import StartReadingModal from '@/components/StartReadingModal.vue';
import { apiFetch } from '@/api/client';
import moment from 'moment';

export default defineComponent({
  components: { ChevronLeftIcon, BookOpenIcon, StartReadingModal },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const book = ref<any>(null);
    const readings = ref<Array<{ id: string; started_at: string; finished_at: string | null; progress: number; total_pages: number }>>([]);
    const loading = ref(true);
    const showStartReadingModal = ref(false);
    const activeTab = ref('Info');
    const tabs = ['Info', 'Log', 'Shelves'];
    const shelves = ref<Array<{ id: string; name: string; description: string }>>([]);
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
          body: JSON.stringify({ book_id: bookId }),
        });
        if (response.ok) {
          const data = await response.json();
          readings.value = data.readings;
          return { googleBooksId: data.google_books_id as string | null, isbn13: data.isbn13 as string | null };
        }
      } catch (error) {
        console.error('Failed to fetch book info:', error);
      }
      return null;
    };

    const fetchBookDetailsWrapper = async (bookId: string) => {
      const info = await fetchBookInfo(bookId);
      if (info?.googleBooksId) {
        book.value = await fetchBookDetails(info.googleBooksId);
      } else if (info?.isbn13) {
        const results = await searchBooks(`isbn:${info.isbn13}`);
        if (results.length > 0) book.value = results[0];
      }
      loading.value = false;
    };

    const fetchShelves = async () => {
      loadingShelves.value = true;
      try {
        const response = await apiFetch('/api/shelves', { method: 'POST' });
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
          showToast('Book added to shelf successfully.', 'alert-success');
        } else {
          showToast('Failed to add book to shelf.', 'alert-error');
        }
      } catch {
        showToast('Failed to add book to shelf.', 'alert-error');
      }
    };

    const viewReadingDetail = (readingId: string) => {
      router.push({ name: 'reading-detail', params: { id: readingId } });
    };

    const startReadingSession = async (totalPages: number) => {
      try {
        const response = await apiFetch('/api/books/start-reading', {
          method: 'POST',
          body: JSON.stringify({ book_id: route.params.id, total_pages: totalPages }),
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
      addBookToShelf,
      formatDate,
    };
  },
});
</script>
