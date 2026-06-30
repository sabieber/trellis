<template>
  <div class="min-h-screen flex flex-col">
    <!-- Header -->
    <div class="flex justify-between items-center px-4 pt-5 pb-2">
      <h1 class="t-display text-2xl">Library</h1>
      <div class="flex items-center gap-2">
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="name">Name</option>
          <option value="created_at">Created at</option>
          <option value="updated_at">Updated at</option>
        </select>
        <CreateShelfModal @shelfCreated="fetchData"/>
      </div>
    </div>

    <div class="px-4 pb-3">
      <p class="t-meta">{{ totalBooks }} books total</p>
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
            <h2 class="t-title text-base leading-tight">{{ shelf.name }}</h2>
            <span class="t-meta">{{ (shelfBooks[shelf.id] || []).length }} books</span>
          </div>
          <div class="flex items-center gap-1">
            <button
                @click.stop="confirmRemoveShelf(shelf.id)"
                class="flex items-center justify-center size-7 rounded-full text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
            >
              <MinusIcon class="size-4"/>
            </button>
            <button
                @click="goToShelf(shelf.id)"
                class="flex items-center t-meta hover:text-ink transition-colors duration-150"
            >
              See all
              <ChevronRightIcon class="size-4 ml-0.5"/>
            </button>
          </div>
        </div>

        <!-- Book tile row -->
        <div class="flex gap-3 pb-1 overflow-hidden">
          <template v-if="(shelfBooks[shelf.id] || []).length > 0">
            <BookCover
                v-for="book in (shelfBooks[shelf.id] || []).slice(0, visibleCount(shelfBooks[shelf.id] || []))"
                :key="book.id"
                :title="book.title"
                :author="book.author"
                :width="tileWidth"
                :cover-url="bookCoverUrl(book)"
                class="cursor-pointer"
                @click="goToBook(book.id)"
            />
            <div
                v-if="(shelfBooks[shelf.id] || []).length > visibleCount(shelfBooks[shelf.id] || [])"
                @click="goToShelf(shelf.id)"
                class="flex-none aspect-2/3 rounded-cover bg-surface border border-line flex items-center justify-center cursor-pointer hover:bg-surface-2 hover:border-line-hair transition-colors duration-150"
                :style="{ width: tileWidth + 'px' }"
            >
              <span class="t-title text-sm text-ink-dim">+{{
                  (shelfBooks[shelf.id] || []).length - visibleCount(shelfBooks[shelf.id] || [])
                }}</span>
            </div>
          </template>
          <div
              v-else
              @click="goToShelf(shelf.id)"
              class="flex-none aspect-2/3 rounded-cover bg-surface border border-dashed border-line flex items-center justify-center cursor-pointer"
              :style="{ width: tileWidth + 'px' }"
          >
            <span class="t-meta text-faint text-center px-1">Empty</span>
          </div>
        </div>
      </div>

      <div v-if="!shelves.length" class="t-meta text-center py-10">No shelves yet.</div>
    </div>

    <ConfirmDialog
        v-if="pendingDeleteShelfId"
        title="Remove Shelf"
        message="Are you sure you want to remove this shelf? Books only on this shelf may be deleted."
        confirmLabel="Remove"
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
import {useRouter} from 'vue-router';
import {MinusIcon, ChevronRightIcon} from '@heroicons/vue/24/outline';
import CreateShelfModal from '@/components/CreateShelfModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import BookCover from '@/components/ui/BookCover.vue';
import {apiFetch} from '@/api/client';

import {bookCoverUrl} from '@/utils/coverUrl';

export default defineComponent({
  components: {CreateShelfModal, ConfirmDialog, MinusIcon, ChevronRightIcon, BookCover},
  setup() {
    const shelves = ref<Array<{
      id: string;
      name: string;
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
    }>>>({});
    const loading = ref(true);
    const router = useRouter();
    const toastMessage = ref('');
    const toastType = ref('');
    const sortBy = ref<'name' | 'created_at' | 'updated_at'>('name');
    const shelvesContainerRef = ref<HTMLElement | null>(null);
    const containerWidth = ref(0);
    const pendingDeleteShelfId = ref<string | null>(null);
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
        arr.sort((a, b) => a.name.localeCompare(b.name));
      } else if (sortBy.value === 'created_at') {
        arr.sort((a, b) => b.created_at.localeCompare(a.created_at));
      } else {
        arr.sort((a, b) => b.updated_at.localeCompare(a.updated_at));
      }
      return arr;
    });

    const totalBooks = computed(() =>
        Object.values(shelfBooks.value).reduce((sum, books) => sum + books.length, 0)
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

    const goToShelf = (shelfId: string) => {
      router.push({name: 'shelf-detail', params: {id: shelfId}});
    };

    const goToBook = (bookId: string) => {
      router.push({name: 'book-detail', params: {id: bookId}});
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
          showToast('Shelf removed.', 'alert-success');
        } else {
          showToast('Failed to remove shelf.', 'alert-error');
        }
      } catch {
        showToast('Failed to remove shelf.', 'alert-error');
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
      goToShelf,
      goToBook,
      confirmRemoveShelf,
      removeShelf,
      bookCoverUrl,
      toastMessage,
      toastType
    };
  },
});
</script>
