<template>
  <div class="min-h-screen bg-gray-900">
  <div class="flex flex-col">
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
          <p class="text-gray-400 text-sm mt-1">By {{ book.volumeInfo.authors?.join(', ') }}</p>
          <p class="text-gray-500 text-xs mt-1">
            {{ book.volumeInfo.publishedDate?.slice(0, 4) }}
            <span v-if="book.volumeInfo.pageCount"> · {{ book.volumeInfo.pageCount }} pages</span>
            <span v-if="book.volumeInfo.categories?.[0]"> · {{ book.volumeInfo.categories[0] }}</span>
          </p>
          <div v-if="book.volumeInfo.averageRating" class="flex items-center gap-1.5 mt-2">
            <template v-for="i in 5" :key="i">
              <StarIcon
                class="size-4"
                :class="i <= Math.round(book.volumeInfo.averageRating) ? 'text-yellow-400' : 'text-gray-600'"
              />
            </template>
            <span class="text-gray-400 text-xs">{{ book.volumeInfo.averageRating }} avg</span>
          </div>
        </div>
      </div>

      <!-- Add to Library CTA -->
      <button @click="scrollToShelfSection" class="btn btn-outline btn-primary w-full mb-6">
        <PlusIcon class="size-5" />
        Add to Library
      </button>

      <!-- Add to Shelf -->
      <div ref="shelfSection" class="mb-6">
        <h2 class="text-xs text-gray-400 font-semibold tracking-widest uppercase mb-3">Add to Shelf</h2>
        <div v-if="loadingShelves" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-md"></span>
        </div>
        <div v-else-if="shelves.length" class="flex flex-col">
          <template v-for="(shelf, index) in shelves" :key="shelf.id">
            <div
              @click="addBookToShelf(shelf.id)"
              class="flex items-center justify-between py-3 cursor-pointer"
            >
              <span class="text-white">{{ shelf.name }}</span>
              <div class="size-7 rounded-full bg-gray-700"></div>
            </div>
            <div v-if="index < shelves.length - 1" class="border-t border-gray-800"></div>
          </template>
        </div>
        <div v-else class="text-gray-500 text-sm py-2">No shelves found.</div>
      </div>

      <!-- Description -->
      <div v-if="book.volumeInfo.description">
        <h2 class="text-xs text-gray-400 font-semibold tracking-widest uppercase mb-3">Description</h2>
        <p class="text-gray-400 text-sm leading-relaxed" v-html="book.volumeInfo.description"></p>
      </div>
    </div>

    <div v-else class="text-gray-500 text-center py-8 px-4">Book not found.</div>
  </div>

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
import { useRoute } from 'vue-router';
import { ChevronLeftIcon, PlusIcon, StarIcon } from "@heroicons/vue/24/solid";
import { fetchBookDetails } from '@/api/googleBooksApi';
import { apiFetch } from '@/api/client';

export default defineComponent({
  components: { ChevronLeftIcon, PlusIcon, StarIcon },
  setup() {
    const route = useRoute();
    const book = ref(null);
    const loading = ref(true);
    const shelves = ref<Array<{ id: string; name: string; description: string }>>([]);
    const loadingShelves = ref(false);
    const shelfSection = ref<HTMLElement | null>(null);
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

    const scrollToShelfSection = () => {
      shelfSection.value?.scrollIntoView({ behavior: 'smooth' });
    };

    onMounted(async () => {
      const bookId = route.params.id as string;
      book.value = await fetchBookDetails(bookId);
      loading.value = false;
      fetchShelves();
    });

    return {
      book,
      loading,
      shelves,
      loadingShelves,
      shelfSection,
      toastMessage,
      toastType,
      addBookToShelf,
      scrollToShelfSection,
    };
  },
});
</script>
