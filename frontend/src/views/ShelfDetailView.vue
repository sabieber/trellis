<template>
  <PageContainer :title="shelf.name" ref="pageContainer">
    <p class="text-sm text-gray-400 mb-4">{{ shelf.description }}</p>
    <div v-if="!loading && books.length" class="flex justify-end mb-3">
      <select v-model="sortBy" class="select select-sm select-bordered w-40">
        <option value="added_at">Added at</option>
        <option value="title">Book name</option>
        <option value="author">Author name</option>
      </select>
    </div>
    <div v-if="loading" class="flex justify-center">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
    <ul v-else-if="books.length" class="space-y-4">
      <li v-for="book in sortedBooks" :key="book.id" class="p-4 bg-gray-700 rounded-lg hover:bg-gray-600 transition cursor-pointer flex gap-3" @click="viewBookDetail(book.id)">
        <img
          v-if="coverUrl(book.google_books_id)"
          :src="coverUrl(book.google_books_id)"
          class="w-12 h-16 object-cover rounded flex-shrink-0 bg-gray-600"
          loading="lazy"
        />
        <div
          v-else
          class="w-12 h-16 rounded flex-shrink-0 flex items-center justify-center"
          :style="{ backgroundColor: getBookColor(book.id) }"
        >
          <span class="text-[8px] text-white/70 text-center leading-tight px-0.5">{{ book.title }}</span>
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex justify-between items-center">
            <h3 class="text-xl font-bold text-white truncate">{{ book.title }}</h3>
            <button @click.stop="removeBookFromShelf(book.id)" class="btn btn-circle btn-sm btn-error flex-shrink-0 ml-2">
              <MinusIcon class="size-3 text-white"/>
            </button>
          </div>
          <p class="text-sm text-gray-400">{{ book.author }}</p>
        </div>
      </li>
    </ul>
    <div v-else class="text-white text-center">No books found.</div>
  </PageContainer>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { MinusIcon } from "@heroicons/vue/16/solid";
import PageContainer from '@/components/PageContainer.vue';
import { apiFetch } from '@/api/client';

const BOOK_COLORS = [
  '#1e3a5f', '#4a2500', '#2d1454', '#3a0f0f', '#0f3d2a',
  '#1a1a4e', '#3d2b00', '#1f3a2a', '#3a1a00', '#0f2a3d',
];

function getBookColor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = ((hash << 5) - hash) + id.charCodeAt(i);
    hash |= 0;
  }
  return BOOK_COLORS[Math.abs(hash) % BOOK_COLORS.length];
}

function coverUrl(googleBooksId: string | null | undefined): string | undefined {
  if (!googleBooksId) return undefined;
  return `https://books.google.com/books/content?id=${googleBooksId}&printsec=frontcover&img=1&zoom=1&source=gbs_api`;
}

export default defineComponent({
  components: { MinusIcon, PageContainer },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const books = ref<Array<{ id: string, title: string, author: string, google_books_id: string | null, added_at: string }>>([]);
    const loading = ref(true);
    const shelf = ref({ name: '', description: '' });
    const pageContainer = ref<any>(null);
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');

    const sortedBooks = computed(() => {
      const arr = [...books.value];
      if (sortBy.value === 'title') {
        arr.sort((a, b) => a.title.localeCompare(b.title));
      } else if (sortBy.value === 'author') {
        arr.sort((a, b) => a.author.localeCompare(b.author));
      } else {
        arr.sort((a, b) => b.added_at.localeCompare(a.added_at));
      }
      return arr;
    });

    const fetchShelfBooks = async (shelfId: string) => {
      try {
        const response = await apiFetch('/api/shelves/books', {
          method: 'POST',
          body: JSON.stringify({ shelf_id: shelfId }),
        });
        if (response.ok) {
          const data = await response.json();
          books.value = data.books;
          shelf.value = data.shelf;
        } else {
          console.error('Failed to fetch books:', await response.json());
        }
      } catch (error) {
        console.error('Failed to fetch books:', error);
      } finally {
        loading.value = false;
      }
    };

    const removeBookFromShelf = async (bookId: string) => {
      try {
        const response = await apiFetch('/api/shelves/remove-book', {
          method: 'POST',
          body: JSON.stringify({ book_id: bookId, shelf_id: route.params.id }),
        });
        if (response.ok) {
          pageContainer.value?.showToast({ message: 'Book removed from shelf successfully.', type: 'alert-success' });
          books.value = books.value.filter((book: { id: string }) => book.id !== bookId);
        } else {
          const data = await response.json();
          console.error('Failed to remove book:', data);
          pageContainer.value?.showToast({ message: data.error || 'Failed to remove book from shelf.', type: 'alert-error' });
        }
      } catch (error) {
        console.error('Failed to remove book:', error);
        pageContainer.value?.showToast({ message: 'Failed to remove book from shelf.', type: 'alert-error' });
      }
    };

    const viewBookDetail = (id: string) => {
      router.push({ name: 'book-detail', params: { id } });
    };

    onMounted(() => fetchShelfBooks(route.params.id as string));

    return { books, sortedBooks, loading, shelf, sortBy, removeBookFromShelf, viewBookDetail, pageContainer, coverUrl, getBookColor };
  },
});
</script>
