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
        <div class="flex flex-col items-center text-center mt-2 mb-6">
          <BookCover
              :title="book.title || 'Untitled'"
              :author="book.authors?.join(', ') || ''"
              :width="128"
              :cover-url="book.cover_url"
          />
          <h1 class="t-display text-[22px] mt-4 max-w-75">{{ book.title }}</h1>
          <p class="t-meta text-sm mt-1.5">{{ book.authors?.join(', ') }}</p>
          <div v-if="book.average_rating" class="flex items-center gap-2 mt-2.5">
            <Stars :rating="book.average_rating"/>
            <span class="t-meta">{{ book.average_rating }} avg</span>
          </div>
          <p class="t-meta mt-2">
            {{ book.published_year }}
            <span v-if="book.page_count"> · {{ book.page_count }} pp</span>
            <span v-if="book.category"> · {{ book.category }}</span>
          </p>
        </div>

        <Button block class="mb-6" @click="scrollToShelfSection">
          <PlusIcon class="size-4"/>
          Add to Library
        </Button>

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

        <div v-if="book.description">
          <h2 class="t-eyebrow mb-2">Description</h2>
          <p class="text-ink-dim text-sm leading-relaxed" v-html="book.description"></p>
        </div>
      </div>

      <div v-else class="t-meta text-center py-8 px-4">Book not found.</div>
    </div>

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
import {fetchBookDetail} from '@/api/bookApi';
import {apiFetch} from '@/api/client';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import Stars from '@/components/ui/Stars.vue';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {ChevronLeftIcon, PlusIcon, BookCover, Button, Stars},
  setup() {
    const route = useRoute();
    const book = ref<BookSearchResult | null>(null);
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
      const compositeId = route.params.id as string;
      const colonIndex = compositeId.indexOf(':');
      const source = compositeId.substring(0, colonIndex);
      const sourceId = compositeId.substring(colonIndex + 1);
      book.value = await fetchBookDetail(source, sourceId);
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
