<template>
  <PageContainer :title="shelf.name" :description="shelf.description" wide ref="pageContainer">
    <template #title-button>
      <div v-if="!loading && books.length" class="flex items-center gap-2">
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="added_at">Added at</option>
          <option value="title">Book name</option>
          <option value="author">Author name</option>
        </select>
        <SegmentedControl v-model="layoutMode" :options="layoutOptions">
          <template #option="{ option }">
            <QueueListIcon v-if="option.value === 'list'" class="size-4"/>
            <Squares2X2Icon v-else class="size-4"/>
          </template>
        </SegmentedControl>
      </div>
    </template>

    <div ref="contentRef">
      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <template v-else-if="books.length">
        <ul v-if="layoutMode === 'list'" class="flex flex-col">
          <li
              v-for="book in sortedBooks"
              :key="book.id"
              class="flex items-center gap-4 py-3 md:py-4 border-b border-line-soft cursor-pointer group"
              @click="viewBookDetail(book.id)"
          >
            <BookCover
                :title="book.title"
                :author="book.author"
                :width="listCoverWidth"
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

        <div v-else class="flex flex-wrap gap-3">
          <BookCover
              v-for="book in sortedBooks"
              :key="book.id"
              :title="book.title"
              :author="book.author"
              :width="gridTileWidth"
              :cover-url="coverUrl(book.google_books_id)"
              class="cursor-pointer"
              @click="viewBookDetail(book.id)"
          />
        </div>
      </template>

      <div v-else class="t-meta text-center py-12">No books found.</div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {MinusIcon, QueueListIcon, Squares2X2Icon} from "@heroicons/vue/24/outline";
import PageContainer from '@/components/PageContainer.vue';
import BookCover from '@/components/ui/BookCover.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import {apiFetch} from '@/api/client';
import {coverUrl} from '@/utils/coverUrl';
import {useContainerWidth} from '@/composables/useContainerWidth';
import moment from 'moment';

const LAYOUT_STORAGE_KEY = 'shelf-layout-mode';
const MD_BREAKPOINT = 768;
const GRID_TILE_SM = 80;
const GRID_TILE_LG = 112;

export default defineComponent({
  components: {MinusIcon, QueueListIcon, Squares2X2Icon, PageContainer, BookCover, SegmentedControl},
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

    const layoutOptions = [
      {value: 'list'},
      {value: 'grid'},
    ];

    const layoutMode = ref(localStorage.getItem(LAYOUT_STORAGE_KEY) === 'grid' ? 'grid' : 'list');
    watch(layoutMode, (val) => localStorage.setItem(LAYOUT_STORAGE_KEY, val));

    const isReady = computed(() => !loading.value);
    const {containerWidth, setupObserver} = useContainerWidth(contentRef, isReady);

    const listCoverWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? 72 : 56
    );

    const gridTileWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? GRID_TILE_LG : GRID_TILE_SM
    );

    const formatAddedAt = (dateStr: string) => moment(dateStr).format('MMM D, YYYY');

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

    return {
      books, sortedBooks, loading, shelf, sortBy,
      layoutMode, layoutOptions,
      listCoverWidth, gridTileWidth,
      pageContainer, contentRef, formatAddedAt, removeBookFromShelf,
      viewBookDetail, coverUrl,
    };
  },
});
</script>
