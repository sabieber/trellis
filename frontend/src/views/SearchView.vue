<template>
  <div class="min-h-screen bg-gray-900">
  <div class="px-4 pt-4">
    <div class="relative mb-6">
      <MagnifyingGlassIcon class="absolute left-4 top-1/2 -translate-y-1/2 size-5 text-gray-400 pointer-events-none" />
      <input
        type="text"
        v-model="query"
        class="input w-full bg-gray-800 border border-gray-700 rounded-full pl-12 text-white placeholder-gray-400 focus:outline-none focus:border-primary"
        placeholder="Search books, authors..."
        @keyup.enter="searchBooks"
      />
    </div>

    <div v-if="loading" class="flex justify-center py-8">
      <span class="loading loading-spinner loading-lg text-primary"></span>
    </div>

    <div v-else-if="books.length">
      <template v-for="(book, index) in books" :key="book.id">
        <div
          class="flex items-center gap-4 py-3 cursor-pointer"
          @click="viewBookDetail(book.id)"
        >
          <img
            :src="book.volumeInfo.imageLinks?.thumbnail"
            alt="Book cover"
            class="w-12 h-16 object-cover rounded flex-shrink-0 bg-gray-700"
          />
          <div class="min-w-0">
            <h3 class="font-bold text-white truncate">{{ book.volumeInfo.title }}</h3>
            <p class="text-gray-400 text-sm truncate">By {{ book.volumeInfo.authors?.join(', ') }}</p>
            <p class="text-gray-500 text-xs mt-0.5">
              {{ book.volumeInfo.publishedDate?.slice(0, 4) }}
              <span v-if="book.volumeInfo.pageCount"> · {{ book.volumeInfo.pageCount }} pages</span>
              <span v-if="book.volumeInfo.categories?.[0]"> · {{ book.volumeInfo.categories[0] }}</span>
            </p>
          </div>
        </div>
        <div v-if="index < books.length - 1" class="border-t border-gray-800"></div>
      </template>
    </div>

    <div v-else class="text-gray-500 text-center py-8">No books found.</div>
  </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { MagnifyingGlassIcon } from "@heroicons/vue/16/solid";
import { searchBooks } from '@/api/googleBooksApi';

export default defineComponent({
  components: { MagnifyingGlassIcon },
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
      router.replace({ query: { q: query.value } });
    };

    const viewBookDetail = (id: string) => {
      router.push({ name: 'search-detail', params: { id } });
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
