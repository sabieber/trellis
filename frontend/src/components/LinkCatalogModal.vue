<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4 max-w-xl">
      <div>
        <h3 class="t-title text-lg">{{ $t('linkCatalog.title') }}</h3>
        <p class="t-meta mt-1">{{ $t('linkCatalog.hint') }}</p>
      </div>

      <!-- A form, not @keydown.enter: Android soft keyboards report keyCode 229
           while composing, so the key never matches there. -->
      <form @submit.prevent="search" class="flex gap-2">
        <div
            class="flex items-center gap-2.5 flex-1 bg-surface border border-line rounded-sm px-3.5 transition-colors duration-150 focus-within:border-green/32"
        >
          <SearchIcon class="size-5 text-muted flex-none"/>
          <input
              ref="input"
              type="text"
              v-model="query"
              class="w-full bg-transparent py-3 text-sm text-ink placeholder:text-muted focus:outline-none"
              :placeholder="$t('search.placeholder')"
          />
        </div>
        <Button type="submit" :disabled="loading">{{ $t('nav.search') }}</Button>
      </form>

      <div v-if="loading" class="flex justify-center py-8">
        <span class="loading loading-spinner loading-md"></span>
      </div>

      <div v-else-if="books.length" class="flex flex-col max-h-96 overflow-y-auto">
        <template v-for="book in books" :key="book.id">
        <div
            class="flex items-center gap-2 border-b border-line-soft group"
        >
          <button
              type="button"
              class="flex gap-3 py-2.5 flex-1 min-w-0 text-left cursor-pointer"
              :disabled="submitting"
              @click="$emit('select', book)"
          >
            <BookCover
                :title="book.title || $t('common.untitled')"
                :author="book.authors?.join(', ') || ''"
                :width="46"
                :cover-url="book.cover_url"
                hoverable
            />
            <div class="min-w-0 flex flex-col justify-center">
              <div class="flex items-center gap-2 min-w-0">
                <h3 class="t-title text-[15px] truncate group-hover:text-green-soft transition-colors duration-150">{{ book.title }}</h3>
                <span class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border border-line text-muted">{{ $t(`search.source.${book.source}`) }}</span>
              </div>
              <p class="t-meta mt-0.5 truncate">{{ book.authors?.join(', ') }}</p>
              <p class="t-meta mt-1">
                {{ book.published_year }}
                <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
                <span v-if="book.category"> · {{ book.category }}</span>
              </p>
            </div>
          </button>
          <button
              v-if="isWork(book)"
              type="button"
              class="flex items-center justify-center size-8 rounded-full flex-none cursor-pointer transition-colors duration-150"
              :class="expanded === book.id ? 'text-green-soft bg-surface-2' : 'text-muted hover:text-ink hover:bg-surface-2'"
              :title="$t('search.editions')"
              :aria-label="$t('search.editions')"
              @click="toggleEditions(book)"
          >
            <ChevronDownIcon class="size-4 transition-transform duration-150" :class="expanded === book.id ? 'rotate-180' : ''"/>
          </button>
          <RouterLink
              :to="{ name: 'search-detail', params: { id: book.id } }"
              target="_blank"
              class="flex items-center justify-center size-8 rounded-full flex-none text-muted hover:text-ink hover:bg-surface-2 transition-colors duration-150"
              :title="$t('linkCatalog.preview')"
              :aria-label="$t('linkCatalog.preview')"
          >
            <ArrowUpRightIcon class="size-4"/>
          </RouterLink>
        </div>

        <div v-if="expanded === book.id" class="flex flex-col bg-surface/60">
          <div v-if="loadingEditions" class="flex justify-center py-4">
            <span class="loading loading-spinner loading-sm"></span>
          </div>
          <p v-else-if="!editions.length" class="t-meta py-3 pl-6">{{ $t('search.noEditions') }}</p>
          <div
              v-for="edition in editions"
              :key="edition.id"
              class="flex items-center gap-2 pl-6 border-b border-line-soft"
          >
            <button
                type="button"
                class="flex-1 min-w-0 text-left py-2 cursor-pointer group/edition"
                :disabled="submitting"
                @click="$emit('select', edition)"
            >
              <div class="flex items-center gap-2 min-w-0">
                <span
                    v-if="edition.language"
                    class="flex-none text-[10px] leading-none uppercase tracking-wide px-1.5 py-0.5 rounded-sm border border-green/40 text-green-soft"
                >{{ edition.language }}</span>
                <span class="text-sm text-ink truncate group-hover/edition:text-green-soft transition-colors duration-150">{{ edition.title }}</span>
              </div>
              <p class="t-meta mt-0.5 truncate">{{ editionMeta(edition) }}</p>
            </button>
            <RouterLink
                :to="{ name: 'search-detail', params: { id: edition.id } }"
                target="_blank"
                class="flex items-center justify-center size-7 rounded-full flex-none text-muted hover:text-ink hover:bg-surface-2 transition-colors duration-150"
                :title="$t('linkCatalog.preview')"
                :aria-label="$t('linkCatalog.preview')"
            >
              <ArrowUpRightIcon class="size-3.5"/>
            </RouterLink>
          </div>
        </div>
        </template>
      </div>

      <p v-else-if="hasSearched" class="t-meta text-center py-6">{{ $t('search.emptyTitle') }}</p>

      <div class="modal-action mt-0">
        <Button variant="ghost" block @click="$emit('close')">{{ $t('common.cancel') }}</Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import {defineComponent, onMounted, ref} from 'vue';
import {SearchIcon, ArrowUpRightIcon, ChevronDownIcon} from '@lucide/vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import {searchBooks} from '@/api/bookApi';
import {useEditions} from '@/composables/useEditions';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {SearchIcon, ArrowUpRightIcon, ChevronDownIcon, BookCover, Button},
  props: {
    initialQuery: {
      type: String,
      required: true,
    },
    // Set while the parent writes the pick, so a second click cannot fire.
    submitting: {
      type: Boolean,
      default: false,
    },
  },
  emits: ['close', 'select'],
  setup(props) {
    const query = ref(props.initialQuery);
    const books = ref<BookSearchResult[]>([]);
    const loading = ref(false);
    const hasSearched = ref(false);
    const {expanded, editions, loadingEditions, isWork, toggleEditions, collapse, editionMeta} =
        useEditions();

    const search = async () => {
      if (!query.value.trim()) return;
      loading.value = true;
      hasSearched.value = true;
      collapse();
      // The user's own rows are what we are linking *from* — only catalog hits
      // can be linked to.
      books.value = (await searchBooks(query.value)).filter((b) => b.source !== 'library');
      loading.value = false;
    };

    onMounted(search);

    return {
      query,
      books,
      loading,
      hasSearched,
      search,
      expanded,
      editions,
      loadingEditions,
      isWork,
      toggleEditions,
      editionMeta,
    };
  },
});
</script>
