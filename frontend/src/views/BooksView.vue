<!-- Every book of the library behind a filter bar. Filtering runs on the
     server (see `/api/books/browse`): this view never holds the whole library,
     it renders whatever the current filter state returns. -->
<template>
  <PageContainer :title="$t('books.title')">
    <!-- Two columns on a phone, one row from `sm` up. -->
    <div class="grid grid-cols-2 gap-2 sm:flex sm:flex-wrap sm:items-center mb-4">
      <FilterSelect
          v-model="shelfId"
          :options="shelfOptions"
          :all-label="$t('books.allShelves')"
          :label="$t('books.filterShelf')"
          :no-match-text="$t('books.noMatch')"
      />
      <FilterSelect
          v-model="author"
          :options="authorOptions"
          :all-label="$t('books.allAuthors')"
          :label="$t('books.filterAuthor')"
          :no-match-text="$t('books.noMatch')"
      />
      <FilterSelect
          v-model="genre"
          :options="genreOptions"
          :all-label="$t('books.allGenres')"
          :label="$t('books.filterGenre')"
          :no-match-text="$t('books.noMatch')"
      />
      <FilterSelect
          v-model="tag"
          :options="tagOptions"
          :all-label="$t('books.allTags')"
          :label="$t('books.filterTag')"
          :no-match-text="$t('books.noMatch')"
      />
      <FilterSelect
          v-model="rating"
          :options="ratingOptions"
          :all-label="$t('books.allRatings')"
          :label="$t('books.filterRating')"
          :no-match-text="$t('books.noMatch')"
      />
      <Button v-if="hasFilters" variant="ghost" class="col-span-2 px-3.5! py-2! text-[13px]!" @click="clearFilters">
        <XIcon class="size-4"/>
        {{ $t('books.clearFilters') }}
      </Button>
    </div>

    <!-- The picker and the mode switch ride on the count row rather than in the
         title bar: a phone has no room next to the title, and they belong to the
         list right below them anyway. -->
    <div class="flex items-center justify-between gap-2 mb-3">
      <p class="t-meta">{{ $t('books.count', { count: total }) }}</p>
      <div v-if="books.length" class="flex items-center gap-2">
        <Button
            variant="ghost"
            icon
            :title="$t('shelf.random')"
            :aria-label="$t('shelf.random')"
            :disabled="pickerLoading"
            @click="openPicker"
        >
          <Dice3Icon class="size-5 shrink-0"/>
        </Button>
        <LayoutModeSelect v-model="layoutMode"/>
      </div>
    </div>

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <template v-else-if="books.length">
      <BookLayout
          :books="books"
          :mode="layoutMode"
      />

      <div v-if="books.length < total" class="flex justify-center py-6">
        <Button variant="soft" :disabled="loadingMore" @click="loadMore">
          <span v-if="loadingMore" class="loading loading-spinner loading-xs"></span>
          {{ $t('books.loadMore', { count: total - books.length }) }}
        </Button>
      </div>
    </template>

    <div v-else class="t-meta text-center py-12">{{ $t('books.noBooks') }}</div>

    <RandomBookModal
        v-if="pickerCandidates.length"
        :books="pickerCandidates"
        @close="pickerCandidates = []"
    />
  </PageContainer>
</template>

<script setup lang="ts">
import {computed, onMounted, ref, watch} from 'vue';
import {useI18n} from 'vue-i18n';
import {useRoute, useRouter} from 'vue-router';
import {Dice3Icon, XIcon} from '@lucide/vue';
import PageContainer from '@/components/PageContainer.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import RandomBookModal from '@/components/shelf/RandomBookModal.vue';
import Button from '@/components/ui/Button.vue';
import FilterSelect from '@/components/ui/FilterSelect.vue';
import {apiFetch} from '@/api/client';
import {useLayoutMode} from '@/composables/useLayoutMode';
import type {ShelfBook} from '@/types/shelf';

const route = useRoute();
const router = useRouter();
const {t} = useI18n();

// Must match `UNLABELLED` in backend/src/books.rs.
const UNLABELLED = '__none__';

const books = ref<ShelfBook[]>([]);
const total = ref(0);
const loading = ref(true);
const loadingMore = ref(false);
const layoutMode = useLayoutMode();
// Non-empty means the picker is open, over exactly these candidates.
const pickerCandidates = ref<ShelfBook[]>([]);
const pickerLoading = ref(false);

const shelves = ref<Array<{ id: string; code: string; name: string | null }>>([]);
const authors = ref<string[]>([]);
const genres = ref<string[]>([]);
const tags = ref<string[]>([]);

// Filter state lives in the URL, so a reload or a return from a book detail
// keeps the filters the user set.
const asQuery = (value: unknown) => (typeof value === 'string' ? value : '');
const shelfId = ref(asQuery(route.query.shelf));
const author = ref(asQuery(route.query.author));
const genre = ref(asQuery(route.query.genre));
const tag = ref(asQuery(route.query.tag));
const rating = ref(asQuery(route.query.rating));

const hasFilters = computed(
    () => !!(shelfId.value || author.value || genre.value || tag.value || rating.value));

// Authors, genres and tags filter on the value the user reads; a shelf filters
// on its id, so only that one needs a label of its own. All four are computed
// rather than mapped in the template, which would hand the dropdowns a fresh
// array on every render of this view and make them re-filter and re-render.
const asOptions = (values: string[]) => values.map((value) => ({value, label: value}));
const shelfOptions = computed(() =>
    shelves.value.map((shelf) => ({value: shelf.id, label: shelf.name || shelf.code})));
const authorOptions = computed(() => asOptions(authors.value));
// The genre and tag lists lead with the "not set yet" entry: the server reads
// this sentinel as "book has no label of this kind", which is how the user works
// through the books that still need one.
const genreOptions = computed(
    () => [{value: UNLABELLED, label: t('books.noGenre')}, ...asOptions(genres.value)]);
const tagOptions = computed(
    () => [{value: UNLABELLED, label: t('books.noTag')}, ...asOptions(tags.value)]);
// The rating scale is fixed at 1..5, so this list needs nothing from the server.
const ratingOptions = computed(() => [
  {value: UNLABELLED, label: t('books.noRating')},
  ...[5, 4, 3, 2, 1].map((value) => ({
    value: String(value),
    label: t('common.ratingAria', {rating: value}),
  })),
]);

// Only the newest request may write the list; changing two filters quickly
// otherwise lets the slower response overwrite the fresher one.
let latestRequest = 0;

// `append` distinguishes the next page from a fresh filter run: a page is added
// to what is on screen, a filter change replaces it.
const fetchBooks = async (append = false) => {
  const request = ++latestRequest;
  if (append) loadingMore.value = true;
  else loading.value = true;
  try {
    const response = await apiFetch('/api/books/browse', {
      method: 'POST',
      body: JSON.stringify({
        shelf_id: shelfId.value,
        author: author.value,
        genre: genre.value,
        tag: tag.value,
        rating: rating.value,
        offset: append ? books.value.length : 0,
      }),
    });
    if (request !== latestRequest) return;
    if (response.ok) {
      const data = await response.json();
      books.value = append ? [...books.value, ...data.books] : data.books;
      total.value = data.total;
      // The server sends the author list with the first page only.
      if (!append) authors.value = data.authors;
    } else {
      console.error('Failed to fetch books:', await response.json());
    }
  } catch (error) {
    console.error('Failed to fetch books:', error);
  } finally {
    if (request === latestRequest) {
      loading.value = false;
      loadingMore.value = false;
    }
  }
};

const loadMore = () => fetchBooks(true);

// The picker draws from the whole filtered set, which only the server knows —
// this view holds just the pages loaded so far, all from the front of the
// alphabet. So ask for the candidates, then open the modal on them.
const openPicker = async () => {
  pickerLoading.value = true;
  try {
    const response = await apiFetch('/api/books/random', {
      method: 'POST',
      body: JSON.stringify({
        shelf_id: shelfId.value,
        author: author.value,
        genre: genre.value,
        tag: tag.value,
        rating: rating.value,
      }),
    });
    if (response.ok) {
      pickerCandidates.value = (await response.json()).books;
    } else {
      console.error('Failed to draw a random book:', await response.json());
    }
  } catch (error) {
    console.error('Failed to draw a random book:', error);
  } finally {
    pickerLoading.value = false;
  }
};

// Shelves and labels come from the endpoints that already serve them elsewhere.
const fetchFilterOptions = async () => {
  const [shelfResponse, labelResponse] = await Promise.all([
    apiFetch('/api/shelves', {method: 'POST'}),
    apiFetch('/api/books/label-suggestions', {method: 'POST'}),
  ]);
  if (shelfResponse.ok) shelves.value = (await shelfResponse.json()).shelves;
  if (labelResponse.ok) {
    const data = await labelResponse.json();
    genres.value = data.genres;
    tags.value = data.tags;
  }
};

const clearFilters = () => {
  shelfId.value = '';
  author.value = '';
  genre.value = '';
  tag.value = '';
  rating.value = '';
};

watch([shelfId, author, genre, tag, rating], () => {
  const query = {
    shelf: shelfId.value,
    author: author.value,
    genre: genre.value,
    tag: tag.value,
    rating: rating.value,
  };
  router.replace({query: Object.fromEntries(Object.entries(query).filter(([, value]) => value))});
  fetchBooks();
});


onMounted(() => {
  fetchFilterOptions();
  fetchBooks();
});
</script>
