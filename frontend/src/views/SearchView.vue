<template>
  <div class="min-h-screen">
    <div class="px-4 pt-5">
      <h1 class="t-display text-2xl mb-4">Search</h1>

      <div
          class="flex items-center gap-2.5 bg-surface border border-line rounded-sm px-3.5 mb-5 transition-colors duration-150 focus-within:border-green/32"
      >
        <MagnifyingGlassIcon class="size-5 text-muted flex-none"/>
        <input
            type="text"
            v-model="query"
            class="w-full bg-transparent py-3 text-sm text-ink placeholder:text-muted focus:outline-none"
            placeholder="Title, author, or ISBN"
            @keyup.enter="searchBooks"
        />
      </div>

      <div v-if="loading" class="flex justify-center py-8">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <div v-else-if="books.length">
        <div
            v-for="book in books"
            :key="book.id"
            class="flex gap-3 py-2.5 border-b border-line-soft cursor-pointer"
            @click="viewBookDetail(book.id)"
        >
          <BookCover
              :title="book.volumeInfo.title || 'Untitled'"
              :author="book.volumeInfo.authors?.join(', ') || ''"
              :width="46"
              :cover-url="book.volumeInfo.imageLinks?.thumbnail"
          />
          <div class="min-w-0 flex flex-col justify-center">
            <h3 class="t-title text-[15px] truncate">{{ book.volumeInfo.title }}</h3>
            <p class="t-meta mt-0.5 truncate">{{ book.volumeInfo.authors?.join(', ') }}</p>
            <p class="t-meta mt-1">
              {{ book.volumeInfo.publishedDate?.slice(0, 4) }}
              <span v-if="book.volumeInfo.pageCount"> · {{ book.volumeInfo.pageCount }} pp</span>
              <span v-if="book.volumeInfo.categories?.[0]"> · {{ book.volumeInfo.categories[0] }}</span>
            </p>
          </div>
        </div>
      </div>

      <div v-else class="flex flex-col items-center text-center pt-12">
        <h3 class="t-title text-[17px]">Nothing's grown here yet</h3>
        <p class="t-meta mt-1.5 max-w-58">Search by title, author, or ISBN to find books.</p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, onMounted} from 'vue';
import {useRouter, useRoute} from 'vue-router';
import {MagnifyingGlassIcon} from "@heroicons/vue/24/outline";
import BookCover from '@/components/ui/BookCover.vue';
import {searchBooks} from '@/api/googleBooksApi';

export default defineComponent({
  components: {MagnifyingGlassIcon, BookCover},
  setup() {
    const query = ref('');
    const books = ref<Array<any>>([]);
    const loading = ref(false);
    const router = useRouter();
    const route = useRoute();

    const searchBooksWrapper = async () => {
      if (!query.value.trim()) return;
      loading.value = true;
      books.value = await searchBooks(query.value);
      loading.value = false;
      router.replace({query: {q: query.value}});
    };

    const viewBookDetail = (id: string) => {
      router.push({name: 'search-detail', params: {id}});
    };

    onMounted(() => {
      const savedQuery = route.query.q as string;
      if (savedQuery) {
        query.value = savedQuery;
        searchBooksWrapper();
      }
    });

    return {
      query,
      books,
      loading,
      searchBooks: searchBooksWrapper,
      viewBookDetail,
    };
  },
});
</script>
