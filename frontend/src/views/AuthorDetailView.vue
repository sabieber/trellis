<template>
  <PageContainer :title="authorName" wide ref="pageContainer">
    <template #title>
      <h2 class="t-display text-2xl truncate">{{ authorName }}</h2>
      <p v-if="!loading && books.length" class="t-meta mt-1">
        <span>{{ $t('author.bookCount', { n: books.length }) }}</span>
        <span v-if="avgRating !== null"> · {{ $t('author.avgRating', { r: avgRating }) }}</span>
        <span v-if="totalPages > 0"> · {{ $t('author.totalPages', { n: totalPages.toLocaleString() }) }}</span>
      </p>
    </template>

    <template #title-button>
      <div v-if="!loading && books.length" class="flex items-center gap-2">
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="added_at">{{ $t('shelf.sortAdded') }}</option>
          <option value="title">{{ $t('shelf.sortTitle') }}</option>
          <option value="author">{{ $t('shelf.sortAuthor') }}</option>
        </select>
        <SegmentedControl v-model="layoutMode" :options="layoutOptions">
          <template #option="{ option }">
            <QueueListIcon v-if="option.value === 'list'" class="size-4"/>
            <Squares2X2Icon v-else-if="option.value === 'grid'" class="size-4"/>
            <BookOpenIcon v-else-if="option.value === 'shelf'" class="size-4"/>
            <RectangleStackIcon v-else class="size-4"/>
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
            @view-author="viewAuthor"
        />
        <ShelfGridView
            v-else-if="layoutMode === 'grid'"
            :books="sortedBooks"
            :tile-width="gridTileWidth"
            @view-book="viewBookDetail"
        />
        <ShelfBoardView
            v-else-if="layoutMode === 'shelf'"
            :books="sortedBooks"
            :spine-height="spineHeight"
            :container-width="containerWidth"
            @view-book="viewBookDetail"
        />
        <ShelfPileView
            v-else
            :books="sortedBooks"
            @view-book="viewBookDetail"
        />
      </template>

      <div v-else class="t-meta text-center py-12">{{ $t('shelf.noBooks') }}</div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {QueueListIcon, Squares2X2Icon, BookOpenIcon, RectangleStackIcon} from "@heroicons/vue/24/outline";
import PageContainer from '@/components/PageContainer.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import ShelfListView from '@/components/shelf/ShelfListView.vue';
import ShelfGridView from '@/components/shelf/ShelfGridView.vue';
import ShelfBoardView from '@/components/shelf/ShelfBoardView.vue';
import ShelfPileView from '@/components/shelf/ShelfPileView.vue';
import {apiFetch} from '@/api/client';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {goToAuthor} from '@/utils/authorRoute';
import type {ShelfBook} from '@/types/shelf';

// Shared with ShelfDetailView so both remember the same layout preference.
const LAYOUT_STORAGE_KEY = 'shelf-layout-mode';
const MD_BREAKPOINT = 768;
const GRID_TILE_SM = 80;
const GRID_TILE_LG = 112;
const SPINE_HEIGHT_SM = 160;
const SPINE_HEIGHT_LG = 200;

export default defineComponent({
  components: {
    QueueListIcon, Squares2X2Icon, BookOpenIcon, RectangleStackIcon,
    PageContainer, SegmentedControl,
    ShelfListView, ShelfGridView, ShelfBoardView, ShelfPileView,
  },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const books = ref<ShelfBook[]>([]);
    const loading = ref(true);
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');
    const pageContainer = ref<any>(null);
    const contentRef = ref<HTMLElement | null>(null);

    const authorName = computed(() => decodeURIComponent(route.params.name as string));

    const layoutOptions = [
      {value: 'list'},
      {value: 'grid'},
      {value: 'shelf'},
      {value: 'pile'},
    ];

    const validLayouts = ['list', 'grid', 'shelf', 'pile'];
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

    const avgRating = computed(() => {
      const rated = books.value.filter((b) => b.rating != null);
      if (!rated.length) return null;
      const sum = rated.reduce((acc, b) => acc + (b.rating as number), 0);
      return (sum / rated.length).toFixed(1);
    });

    const totalPages = computed(() =>
        books.value.reduce((acc, b) => acc + (b.page_count || 0), 0)
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

    const fetchAuthorBooks = async (author: string) => {
      loading.value = true;
      try {
        const response = await apiFetch('/api/authors/books', {
          method: 'POST',
          body: JSON.stringify({author}),
        });
        if (response.ok) {
          const data = await response.json();
          books.value = data.books;
        } else {
          console.error('Failed to fetch author books:', await response.json());
        }
      } catch (error) {
        console.error('Failed to fetch author books:', error);
      } finally {
        loading.value = false;
      }
    };

    const viewBookDetail = (id: string) => {
      router.push({name: 'book-detail', params: {id}});
    };

    const viewAuthor = (author: string) => goToAuthor(router, author);

    onMounted(() => fetchAuthorBooks(authorName.value));
    // Re-fetch when navigating between authors without unmounting the view.
    watch(authorName, (name) => fetchAuthorBooks(name));

    return {
      books, sortedBooks, loading, sortBy, authorName,
      avgRating, totalPages,
      layoutMode, layoutOptions,
      listCoverWidth, gridTileWidth, spineHeight, containerWidth,
      pageContainer, contentRef,
      viewBookDetail, viewAuthor,
    };
  },
});
</script>
