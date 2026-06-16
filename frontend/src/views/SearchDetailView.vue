<template>
  <div class="min-h-screen">
    <div class="flex flex-col">
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
        <!-- Centered hero -->
        <div class="flex flex-col items-center text-center mt-2 mb-6">
          <BookCover
              :title="book.volumeInfo.title || 'Untitled'"
              :author="book.volumeInfo.authors?.join(', ') || ''"
              :width="128"
              :cover-url="book.volumeInfo.imageLinks?.thumbnail"
          />
          <h1 class="t-display text-[22px] mt-4 max-w-75">{{ book.volumeInfo.title }}</h1>
          <p class="t-meta text-sm mt-1.5">{{ book.volumeInfo.authors?.join(', ') }}</p>
          <div v-if="book.volumeInfo.averageRating" class="flex items-center gap-2 mt-2.5">
            <Stars :rating="book.volumeInfo.averageRating"/>
            <span class="t-meta">{{ book.volumeInfo.averageRating }} avg</span>
          </div>
          <p class="t-meta mt-2">
            {{ book.volumeInfo.publishedDate?.slice(0, 4) }}
            <span v-if="book.volumeInfo.pageCount"> · {{ book.volumeInfo.pageCount }} pp</span>
            <span v-if="book.volumeInfo.categories?.[0]"> · {{ book.volumeInfo.categories[0] }}</span>
          </p>
        </div>

        <!-- Add to Library CTA -->
        <Button block class="mb-6" @click="scrollToShelfSection">
          <PlusIcon class="size-4"/>
          Add to Library
        </Button>

        <!-- Add to Shelf -->
        <div ref="shelfSection" class="mb-6">
          <h2 class="t-eyebrow mb-2">Add to shelf</h2>
          <div v-if="loadingShelves" class="flex justify-center py-4">
            <span class="loading loading-spinner loading-md"></span>
          </div>
          <div v-else-if="shelves.length" class="flex flex-col">
            <div
                v-for="shelf in shelves"
                :key="shelf.id"
                @click="addBookToShelf(shelf.id)"
                class="flex items-center justify-between py-3 border-b border-line-soft cursor-pointer group"
            >
              <span class="text-sm text-ink group-hover:text-green-soft transition-colors duration-150">{{
                  shelf.name
                }}</span>
              <PlusIcon class="size-4 text-muted group-hover:text-green-soft transition-colors duration-150"/>
            </div>
          </div>
          <div v-else class="t-meta py-2">No shelves found.</div>
        </div>

        <!-- Description -->
        <div v-if="book.volumeInfo.description">
          <h2 class="t-eyebrow mb-2">Description</h2>
          <p class="text-ink-dim text-sm leading-relaxed" v-html="book.volumeInfo.description"></p>
        </div>
      </div>

      <div v-else class="t-meta text-center py-8 px-4">Book not found.</div>
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
import {defineComponent, ref, onMounted} from 'vue';
import {useRoute} from 'vue-router';
import {ChevronLeftIcon, PlusIcon} from "@heroicons/vue/24/outline";
import {fetchBookDetails} from '@/api/googleBooksApi';
import {apiFetch} from '@/api/client';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import Stars from '@/components/ui/Stars.vue';

export default defineComponent({
  components: {ChevronLeftIcon, PlusIcon, BookCover, Button, Stars},
  setup() {
    const route = useRoute();
    const book = ref<any>(null);
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

    const scrollToShelfSection = () => {
      shelfSection.value?.scrollIntoView({behavior: 'smooth'});
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
