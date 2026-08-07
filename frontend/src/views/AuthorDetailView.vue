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
        <LayoutModeSelect v-model="layoutMode"/>
      </div>
    </template>

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <BookLayout
        v-else-if="books.length"
        :books="sortedBooks"
        :mode="layoutMode"
        @view-book="viewBookDetail"
        @view-author="viewAuthor"
    />

    <div v-else class="t-meta text-center py-12">{{ $t('shelf.noBooks') }}</div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import PageContainer from '@/components/PageContainer.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import {apiFetch} from '@/api/client';
import {useLayoutMode} from '@/composables/useLayoutMode';
import {goToAuthor} from '@/utils/authorRoute';
import type {ShelfBook} from '@/types/shelf';

export default defineComponent({
  components: {
    PageContainer, BookLayout, LayoutModeSelect,
  },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const books = ref<ShelfBook[]>([]);
    const loading = ref(true);
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');
    const pageContainer = ref<any>(null);
    const layoutMode = useLayoutMode();

    const authorName = computed(() => decodeURIComponent(route.params.name as string));

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
      avgRating, totalPages, layoutMode, pageContainer,
      viewBookDetail, viewAuthor,
    };
  },
});
</script>
