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
      <BookResultRow v-for="book in books" :key="book.id" :book="book"/>
    </div>

    <div v-else class="t-meta text-center py-12">{{ $t('series.empty') }}</div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute} from 'vue-router';
import PageContainer from '@/components/PageContainer.vue';
import BookResultRow from '@/components/ui/BookResultRow.vue';
import {fetchSeries} from '@/api/bookApi';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {PageContainer, BookResultRow},
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
