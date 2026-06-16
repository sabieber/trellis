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
            <Squares2X2Icon v-else-if="option.value === 'grid'" class="size-4"/>
            <BookOpenIcon v-else class="size-4"/>
          </template>
        </SegmentedControl>
      </div>
    </template>

    <div ref="contentRef">
      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <template v-else-if="books.length">
        <ShelfListView
            v-if="layoutMode === 'list'"
            :books="sortedBooks"
            :cover-width="listCoverWidth"
            @view-book="viewBookDetail"
            @remove-book="removeBookFromShelf"
        />
        <ShelfGridView
            v-else-if="layoutMode === 'grid'"
            :books="sortedBooks"
            :tile-width="gridTileWidth"
            @view-book="viewBookDetail"
        />
        <ShelfBoardView
            v-else
            :books="sortedBooks"
            :spine-height="spineHeight"
            :container-width="containerWidth"
            @view-book="viewBookDetail"
        />
      </template>

      <div v-else class="t-meta text-center py-12">No books found.</div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {QueueListIcon, Squares2X2Icon, BookOpenIcon} from "@heroicons/vue/24/outline";
import PageContainer from '@/components/PageContainer.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import ShelfListView from '@/components/shelf/ShelfListView.vue';
import ShelfGridView from '@/components/shelf/ShelfGridView.vue';
import ShelfBoardView from '@/components/shelf/ShelfBoardView.vue';
import {apiFetch} from '@/api/client';
import {useContainerWidth} from '@/composables/useContainerWidth';
import type {ShelfBook} from '@/types/shelf';

const LAYOUT_STORAGE_KEY = 'shelf-layout-mode';
const MD_BREAKPOINT = 768;
const GRID_TILE_SM = 80;
const GRID_TILE_LG = 112;
const SPINE_HEIGHT_SM = 160;
const SPINE_HEIGHT_LG = 200;

export default defineComponent({
  components: {
    QueueListIcon, Squares2X2Icon, BookOpenIcon,
    PageContainer, SegmentedControl,
    ShelfListView, ShelfGridView, ShelfBoardView,
  },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const books = ref<ShelfBook[]>([]);
    const loading = ref(true);
    const shelf = ref({name: '', description: ''});
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');
    const pageContainer = ref<any>(null);
    const contentRef = ref<HTMLElement | null>(null);

    const layoutOptions = [
      {value: 'list'},
      {value: 'grid'},
      {value: 'shelf'},
    ];

    const validLayouts = ['list', 'grid', 'shelf'];
    const saved = localStorage.getItem(LAYOUT_STORAGE_KEY);
    const layoutMode = ref(validLayouts.includes(saved || '') ? saved! : 'list');
    watch(layoutMode, (val) => localStorage.setItem(LAYOUT_STORAGE_KEY, val));

    const isReady = computed(() => !loading.value);
    const {containerWidth} = useContainerWidth(contentRef, isReady);

    const listCoverWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? 72 : 56
    );

    const gridTileWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? GRID_TILE_LG : GRID_TILE_SM
    );

    const spineHeight = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? SPINE_HEIGHT_LG : SPINE_HEIGHT_SM
    );

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
          books.value = books.value.filter((book) => book.id !== bookId);
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
      listCoverWidth, gridTileWidth, spineHeight, containerWidth,
      pageContainer, contentRef, removeBookFromShelf, viewBookDetail,
    };
  },
});
</script>
