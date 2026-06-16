<template>
  <PageContainer :title="shelf.name" :description="shelf.description" wide ref="pageContainer">
    <template #title-button>
      <div v-if="!loading && books.length" class="flex-none pt-1">
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="added_at">Added at</option>
          <option value="title">Book name</option>
          <option value="author">Author name</option>
        </select>
      </div>
    </template>

    <div ref="contentRef">
      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <ul v-else-if="books.length" class="flex flex-col">
        <li
            v-for="book in sortedBooks"
            :key="book.id"
            class="flex items-center gap-4 py-3 md:py-4 border-b border-line-soft cursor-pointer group"
            @click="viewBookDetail(book.id)"
        >
          <BookCover
              :title="book.title"
              :author="book.author"
              :width="coverWidth"
              :cover-url="coverUrl(book.google_books_id)"
          />
          <div class="flex-1 min-w-0 flex flex-col justify-center">
            <h3 class="t-title text-[15px] md:text-base truncate">{{ book.title }}</h3>
            <p class="t-meta mt-0.5">{{ book.author }}</p>
            <p class="t-mono mt-1 hidden md:block">Added {{ formatAddedAt(book.added_at) }}</p>
          </div>
          <button
              @click.stop="removeBookFromShelf(book.id)"
              class="self-center flex-none flex items-center justify-center size-8 md:size-9 rounded-full text-muted hover:text-ink hover:bg-surface-2 transition-colors duration-150"
          >
            <MinusIcon class="size-4 md:size-5"/>
          </button>
        </li>
      </ul>

      <div v-else class="t-meta text-center py-12">No books found.</div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, onUnmounted, nextTick, watch} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {MinusIcon} from "@heroicons/vue/24/outline";
import PageContainer from '@/components/PageContainer.vue';
import BookCover from '@/components/ui/BookCover.vue';
import {apiFetch} from '@/api/client';
import moment from 'moment';

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
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');
    const pageContainer = ref<any>(null);
    const contentRef = ref<HTMLElement | null>(null);
    const containerWidth = ref(0);
    let resizeObserver: ResizeObserver | null = null;

    const MD_BREAKPOINT = 768;

    const coverWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? 72 : 56
    );

    const formatAddedAt = (dateStr: string) => moment(dateStr).format('MMM D, YYYY');

    const setupResizeObserver = () => {
      if (contentRef.value && !resizeObserver) {
        containerWidth.value = contentRef.value.clientWidth;
        resizeObserver = new ResizeObserver((entries) => {
          for (const entry of entries) {
            containerWidth.value = entry.contentRect.width;
          }
        });
        resizeObserver.observe(contentRef.value);
      }
    };

    watch(loading, (newVal) => {
      if (!newVal) nextTick(setupResizeObserver);
    });

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

    onUnmounted(() => {
      resizeObserver?.disconnect();
    });

    return {
      books, sortedBooks, loading, shelf, sortBy, coverWidth,
      pageContainer, contentRef, formatAddedAt, removeBookFromShelf,
      viewBookDetail, coverUrl,
    };
  },
});
</script>
