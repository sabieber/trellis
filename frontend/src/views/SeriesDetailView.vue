<template>
  <PageContainer :title="seriesName">
    <template #title>
      <h2 class="t-display text-2xl truncate">{{ seriesName }}</h2>
      <p v-if="!loading && books.length" class="t-meta mt-1">
        {{ $t('series.bookCount', { n: books.length }) }}
      </p>
    </template>

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <div v-else-if="books.length">
      <!-- Series members are external catalog hits — link to the
           external-lookup detail view (mirrors SearchView), not book-detail. -->
      <RouterLink
          v-for="book in books"
          :key="book.id"
          class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer group"
          :to="{ name: 'search-detail', params: { id: book.id } }"
      >
        <BookCover
            :title="book.title || $t('common.untitled')"
            :author="book.authors?.join(', ') || ''"
            :width="46"
            :cover-url="book.cover_url"
            hoverable
        />
        <div class="min-w-0 flex flex-col justify-center">
          <h3 class="t-title text-[15px] truncate group-hover:text-green-soft transition-colors duration-150">{{ book.title }}</h3>
          <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
          <p class="t-meta mt-1">
            {{ book.published_year }}
            <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
          </p>
        </div>
      </RouterLink>
    </div>

    <div v-else class="t-meta text-center py-12">{{ $t('series.empty') }}</div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute} from 'vue-router';
import PageContainer from '@/components/PageContainer.vue';
import BookCover from '@/components/ui/BookCover.vue';
import {fetchSeries} from '@/api/bookApi';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {PageContainer, BookCover},
  setup() {
    const route = useRoute();
    const books = ref<BookSearchResult[]>([]);
    const seriesName = ref('');
    const loading = ref(true);

    const seriesKey = computed(() => route.params.key as string);

    const load = async (key: string) => {
      loading.value = true;
      const data = await fetchSeries(key);
      seriesName.value = data?.name ?? '';
      books.value = data?.books ?? [];
      loading.value = false;
    };

    onMounted(() => load(seriesKey.value));
    watch(seriesKey, (key) => load(key));

    return {books, seriesName, loading};
  },
});
</script>
