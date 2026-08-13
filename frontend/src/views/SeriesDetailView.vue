<template>
  <PageContainer :title="seriesName">
    <template #title>
      <h2 class="t-display text-2xl truncate">{{ seriesName }}</h2>
      <p v-if="!loading && books.length" class="t-meta mt-1">
        {{ $t('series.bookCount', { n: books.length }) }}
      </p>
    </template>

    <template #title-button>
      <LayoutModeSelect v-if="!loading && books.length" v-model="layoutMode"/>
    </template>

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <div v-else-if="books.length">
      <!-- Series members are external catalog hits — link to the
           external-lookup detail view (mirrors SearchView), not book-detail. -->
      <template v-if="layoutMode === 'list'">
        <BookResultRow v-for="book in books" :key="book.id" :book="book"/>
      </template>
      <BookLayout v-else :books="layoutBooks" :mode="layoutMode"/>
    </div>

    <div v-else class="t-meta text-center py-12">{{ $t('series.empty') }}</div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute} from 'vue-router';
import PageContainer from '@/components/PageContainer.vue';
import BookResultRow from '@/components/ui/BookResultRow.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import {fetchSeries} from '@/api/bookApi';
import {asShelfBook} from '@/utils/catalogBook';
import {useLayoutMode} from '@/composables/useLayoutMode';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {PageContainer, BookResultRow, BookLayout, LayoutModeSelect},
  setup() {
    const layoutMode = useLayoutMode();
    const route = useRoute();
    const books = ref<BookSearchResult[]>([]);
    const seriesName = ref('');
    const loading = ref(true);

    const seriesKey = computed(() => route.params.key as string);
    const layoutBooks = computed(() => books.value.map(asShelfBook));

    const load = async (key: string) => {
      loading.value = true;
      const data = await fetchSeries(key);
      seriesName.value = data?.name ?? '';
      books.value = data?.books ?? [];
      loading.value = false;
    };

    onMounted(() => load(seriesKey.value));
    watch(seriesKey, (key) => load(key));

    return {books, seriesName, loading, layoutMode, layoutBooks};
  },
});
</script>
