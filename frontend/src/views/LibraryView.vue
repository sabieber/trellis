<template>
  <div class="min-h-screen flex flex-col">
    <!-- Header: title, subtitle under it at `mt-1` like every other screen, and
         the primary action. A phone fits nothing else on this row. -->
    <div class="flex justify-between items-start gap-2 px-4 pt-5 pb-3">
      <div class="min-w-0">
        <h1 class="t-display text-2xl">{{ $t('nav.library') }}</h1>
        <!-- The book count doubles as the link to the browse view, so the
             header carries no extra button. -->
        <RouterLink
            :to="{ name: 'books' }"
            class="t-meta mt-1 flex items-center gap-1 cursor-pointer hover:text-ink transition-colors duration-150"
            :aria-label="$t('library.allBooks')"
        >
          {{ $t('library.booksTotal', { count: totalBooks }) }}
          <ChevronRightIcon class="size-4"/>
        </RouterLink>
      </div>
      <div class="flex flex-wrap justify-end gap-2 shrink-0">
        <CreateBookModal @bookCreated="fetchData"/>
        <CreateShelfModal @shelfCreated="fetchData"/>
      </div>
    </div>

    <!-- Sorting belongs to the shelf list below, not to the title block. -->
    <div class="px-4 pb-3 flex justify-end">
      <select v-model="sortBy" class="select w-36">
        <option value="name">{{ $t('library.sortName') }}</option>
        <option value="created_at">{{ $t('library.sortCreated') }}</option>
        <option value="updated_at">{{ $t('library.sortUpdated') }}</option>
      </select>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-10">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <!-- Shelves -->
    <div v-else ref="shelvesContainerRef" class="flex flex-col gap-7 pb-4">
      <div v-for="shelf in sortedShelves" :key="shelf.id" class="px-4">
        <!-- Shelf header row -->
        <div class="flex justify-between items-center mb-3">
          <div class="flex items-baseline gap-2">
            <h2 class="t-title text-base leading-tight">{{ shelf.name || shelf.code }}</h2>
            <span class="t-meta">{{ $t('library.bookCount', { count: (shelfBooks[shelf.id] || []).length }) }}</span>
          </div>
          <div class="flex items-center gap-1">
            <Button variant="ghost" class="px-2! py-2! text-[13px]!" @click="confirmRemoveShelf(shelf.id)">
              <MinusIcon class="size-4"/>
            </Button>
            <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" :to="{ name: 'shelf-detail', params: { id: shelf.id } }">
              {{ $t('common.seeAll') }}
              <ChevronRightIcon class="size-4"/>
            </Button>
          </div>
        </div>

        <!-- Book tile row -->
        <div class="flex gap-3 -m-1 p-2 overflow-hidden">
          <template v-if="(shelfBooks[shelf.id] || []).length > 0">
            <RouterLink
                v-for="book in (shelfBooks[shelf.id] || []).slice(0, visibleCount(shelfBooks[shelf.id] || []))"
                :key="book.id"
                :to="{ name: 'book-detail', params: { id: book.id } }"
            >
              <BookCover
                  :title="book.title"
                  :author="book.author"
                  :width="tileWidth"
                  :cover-url="resolvedCoverUrl(book.id, bookCoverUrl(book))"
                  :book-id="book.id"
                  @resolve-cover="onResolveCover"
                  :rating="book.rating"
                  :has-note="book.has_notes"
                  hoverable
              />
            </RouterLink>
            <RouterLink
                v-if="(shelfBooks[shelf.id] || []).length > visibleCount(shelfBooks[shelf.id] || [])"
                :to="{ name: 'shelf-detail', params: { id: shelf.id } }"
                class="flex-none aspect-2/3 rounded-cover bg-surface border border-line flex items-center justify-center cursor-pointer hoverable-card"
                :style="{ width: tileWidth + 'px' }"
            >
              <span class="t-title text-sm text-ink-dim">+{{
                  (shelfBooks[shelf.id] || []).length - visibleCount(shelfBooks[shelf.id] || [])
                }}</span>
            </RouterLink>
          </template>
          <RouterLink
              v-else
              :to="{ name: 'shelf-detail', params: { id: shelf.id } }"
              class="flex-none aspect-2/3 rounded-cover bg-surface border border-dashed border-line flex items-center justify-center cursor-pointer hoverable-card"
              :style="{ width: tileWidth + 'px' }"
          >
            <span class="t-meta text-faint text-center px-1">{{ $t('library.empty') }}</span>
          </RouterLink>
        </div>
      </div>

      <div v-if="!shelves.length" class="t-meta text-center py-10">{{ $t('library.noShelves') }}</div>
    </div>

    <ConfirmDialog
        v-if="pendingDeleteShelfId"
        :title="$t('library.removeShelfTitle')"
        :message="$t('library.removeShelfMessage')"
        :confirmLabel="$t('common.remove')"
        @confirm="removeShelf"
        @cancel="pendingDeleteShelfId = null"
    />

    <!-- Toast -->
    <div v-if="toastMessage" class="toast toast-top toast-center pt-16">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, onUnmounted, watch, nextTick} from 'vue';
import {useI18n} from 'vue-i18n';
import {MinusIcon, ChevronRightIcon} from '@lucide/vue';
import CreateShelfModal from '@/components/CreateShelfModal.vue';
import CreateBookModal from '@/components/CreateBookModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import {apiFetch} from '@/api/client';
import {apiErrorMessage} from '@/utils/apiError';

import {bookCoverUrl} from '@/utils/coverUrl';
import {useBookCovers} from '@/composables/useBookCovers';

export default defineComponent({
  components: {CreateShelfModal, CreateBookModal, ConfirmDialog, MinusIcon, ChevronRightIcon, BookCover, Button},
  setup() {
    const {t} = useI18n();
    const shelves = ref<Array<{
      id: string;
      code: string;
      name: string | null;
      description: string;
      created_at: string;
      updated_at: string
    }>>([]);
    const shelfBooks = ref<Record<string, Array<{
      id: string;
      title: string;
      author: string;
      isbn13: string | null;
      isbn10: string | null;
      google_books_id: string | null;
      open_library_id: string | null;
      rating: number | null;
      cover_url: string | null;
      has_notes: boolean;
    }>>>({});
    const loading = ref(true);
    const toastMessage = ref('');
    const toastType = ref('');
    const sortBy = ref<'name' | 'created_at' | 'updated_at'>('name');
    const shelvesContainerRef = ref<HTMLElement | null>(null);
    const containerWidth = ref(0);
    const pendingDeleteShelfId = ref<string | null>(null);
    const { resolvedCoverUrl, onResolveCover } = useBookCovers();
    let resizeObserver: ResizeObserver | null = null;

    const TILE_W_SM = 80;
    const TILE_W_LG = 112;
    const GAP = 12;
    const SM_BREAKPOINT = 640;

    const tileWidth = computed(() =>
        containerWidth.value >= SM_BREAKPOINT ? TILE_W_LG : TILE_W_SM
    );

    const visibleCount = (books: Array<unknown>): number => {
      const w = containerWidth.value - 32;
      if (w <= 0) return 4;
      const maxFit = Math.floor((w + GAP) / (tileWidth.value + GAP));
      if (books.length <= maxFit) return books.length;
      return Math.max(0, maxFit - 1);
    };

    const setupResizeObserver = () => {
      if (shelvesContainerRef.value && !resizeObserver) {
        containerWidth.value = shelvesContainerRef.value.clientWidth;
        resizeObserver = new ResizeObserver((entries) => {
          for (const entry of entries) {
            containerWidth.value = entry.contentRect.width;
          }
        });
        resizeObserver.observe(shelvesContainerRef.value);
      }
    };

    watch(loading, (newVal) => {
      if (!newVal) nextTick(setupResizeObserver);
    });

    const sortedShelves = computed(() => {
      const arr = [...shelves.value];
      if (sortBy.value === 'name') {
        arr.sort((a, b) => (a.name || a.code).localeCompare(b.name || b.code));
      } else if (sortBy.value === 'created_at') {
        arr.sort((a, b) => b.created_at.localeCompare(a.created_at));
      } else {
        arr.sort((a, b) => b.updated_at.localeCompare(a.updated_at));
      }
      return arr;
    });

    // Distinct books, not the sum over shelves: a book on three shelves is one
    // book, and this number now links to the browse view, which counts the same way.
    const totalBooks = computed(() =>
        new Set(Object.values(shelfBooks.value).flat().map((book) => book.id)).size
    );

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    const fetchData = async () => {
      loading.value = true;
      try {
        const res = await apiFetch('/api/shelves', {method: 'POST'});
        if (!res.ok) return;
        const data = await res.json();
        shelves.value = data.shelves;

        const bookResults = await Promise.all(
            data.shelves.map((shelf: { id: string }) =>
                apiFetch('/api/shelves/books', {
                  method: 'POST',
                  body: JSON.stringify({shelf_id: shelf.id}),
                }).then(r => r.ok ? r.json() : {books: []})
            )
        );

        const map: Record<string, Array<{
          id: string;
          title: string;
          author: string;
          isbn13: string | null;
          isbn10: string | null;
          google_books_id: string | null;
          open_library_id: string | null;
          rating: number | null;
          cover_url: string | null;
          has_notes: boolean;
        }>> = {};
        data.shelves.forEach((shelf: { id: string }, i: number) => {
          map[shelf.id] = bookResults[i].books;
        });
        shelfBooks.value = map;
      } catch (error) {
        console.error('Failed to fetch library:', error);
      } finally {
        loading.value = false;
      }
    };

    const confirmRemoveShelf = (shelfId: string) => {
      pendingDeleteShelfId.value = shelfId;
    };

    const removeShelf = async () => {
      const shelfId = pendingDeleteShelfId.value;
      if (!shelfId) return;
      pendingDeleteShelfId.value = null;
      try {
        const res = await apiFetch('/api/shelves/remove', {
          method: 'POST',
          body: JSON.stringify({shelf_id: shelfId}),
        });
        if (res.ok) {
          shelves.value = shelves.value.filter(s => s.id !== shelfId);
          const map = {...shelfBooks.value};
          delete map[shelfId];
          shelfBooks.value = map;
          showToast(t('library.shelfRemoved'), 'alert-success');
        } else {
          showToast(apiErrorMessage(res.status, t), 'alert-error');
        }
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
    };

    onMounted(fetchData);

    onUnmounted(() => {
      resizeObserver?.disconnect();
    });

    return {
      shelves,
      sortedShelves,
      sortBy,
      shelfBooks,
      loading,
      totalBooks,
      shelvesContainerRef,
      tileWidth,
      visibleCount,
      pendingDeleteShelfId,
      fetchData,
      confirmRemoveShelf,
      removeShelf,
      bookCoverUrl,
      resolvedCoverUrl,
      onResolveCover,
      toastMessage,
      toastType
    };
  },
});
</script>
