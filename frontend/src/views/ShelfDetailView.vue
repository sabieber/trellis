<template>
  <PageContainer :title="shelf.name" ref="pageContainer">
    <p v-if="shelf.description" class="t-meta mb-4">{{ shelf.description }}</p>
    <div v-if="!loading && books.length" class="flex justify-end mb-3">
      <select v-model="sortBy" class="select select-sm w-36">
        <option value="added_at">Added at</option>
        <option value="title">Book name</option>
        <option value="author">Author name</option>
      </select>
    </div>
    <div v-if="loading" class="flex justify-center">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
    <ul v-else-if="books.length">
      <li
          v-for="book in sortedBooks"
          :key="book.id"
          class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer group"
          @click="viewBookDetail(book.id)"
      >
        <BookCover
            :title="book.title"
            :author="book.author"
            :width="46"
            :cover-url="coverUrl(book.google_books_id)"
        />
        <div class="flex-1 min-w-0 flex flex-col justify-center">
          <h3 class="t-title text-[15px] truncate">{{ book.title }}</h3>
          <p class="t-meta mt-0.5">{{ book.author }}</p>
        </div>
        <button
            @click.stop="removeBookFromShelf(book.id)"
            class="self-center flex items-center justify-center size-8 rounded-full text-muted hover:text-ink hover:bg-surface-2 transition-colors duration-150"
        >
          <MinusIcon class="size-4"/>
        </button>
      </li>
    </ul>
    <div v-else class="t-meta text-center py-8">No books found.</div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {MinusIcon} from "@heroicons/vue/24/outline";
import PageContainer from '@/components/PageContainer.vue';
import BookCover from '@/components/ui/BookCover.vue';
import {apiFetch} from '@/api/client';

function coverUrl(googleBooksId: string | null | undefined): string | undefined {
  if (!googleBooksId) return undefined;
  return `https://books.google.com/books/content?id=${googleBooksId}&printsec=frontcover&img=1&zoom=1&source=gbs_api`;
}

export default defineComponent({
  components: {MinusIcon, PageContainer, BookCover},
  setup() {
    const route = useRoute();
    const router = useRouter();
    const books = ref<Array<{
      id: string,
      title: string,
      author: string,
      google_books_id: string | null,
      added_at: string
    }>>([]);
    const loading = ref(true);
    const shelf = ref({name: '', description: ''});
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
          body: JSON.stringify({shelf_id: shelfId}),
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
          body: JSON.stringify({book_id: bookId, shelf_id: route.params.id}),
        });
        if (response.ok) {
          pageContainer.value?.showToast({message: 'Book removed from shelf successfully.', type: 'alert-success'});
          books.value = books.value.filter((book: { id: string }) => book.id !== bookId);
        } else {
          const data = await response.json();
          console.error('Failed to remove book:', data);
          pageContainer.value?.showToast({
            message: data.error || 'Failed to remove book from shelf.',
            type: 'alert-error'
          });
        }
      } catch (error) {
        console.error('Failed to remove book:', error);
        pageContainer.value?.showToast({message: 'Failed to remove book from shelf.', type: 'alert-error'});
      }
    };

    const viewBookDetail = (id: string) => {
      router.push({name: 'book-detail', params: {id}});
    };

    onMounted(() => fetchShelfBooks(route.params.id as string));

    return {books, sortedBooks, loading, shelf, sortBy, removeBookFromShelf, viewBookDetail, pageContainer, coverUrl};
  },
});
</script>
