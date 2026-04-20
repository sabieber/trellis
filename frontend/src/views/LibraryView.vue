<template>
  <div class="min-h-screen bg-base-300 flex flex-col">
    <!-- Header -->
    <div class="flex justify-between items-center px-4 pt-5 pb-2">
      <h1 class="text-2xl font-bold">Library</h1>
      <CreateShelfModal @shelfCreated="fetchData" />
    </div>

    <!-- User section -->
    <div class="flex items-center gap-3 px-4 py-3">
      <div class="avatar placeholder">
        <div class="w-12 rounded-full bg-teal-600 flex items-center justify-center">
          <UserIcon class="size-6 text-white" />
        </div>
      </div>
      <div>
        <p class="text-sm opacity-50">{{ totalBooks }} books total</p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-10">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <!-- Shelves -->
    <div v-else class="flex flex-col gap-6 pb-4">
      <div v-for="shelf in shelves" :key="shelf.id" class="px-4">
        <!-- Shelf header row -->
        <div class="flex justify-between items-center mb-3">
          <div class="flex items-baseline gap-2">
            <h2 class="font-bold text-lg leading-tight">{{ shelf.name }}</h2>
            <span class="text-sm opacity-50">{{ (shelfBooks[shelf.id] || []).length }} books</span>
          </div>
          <div class="flex items-center gap-1">
            <button @click.stop="removeShelf(shelf.id)" class="btn btn-ghost btn-xs text-error opacity-60">
              <MinusIcon class="size-4" />
            </button>
            <button @click="goToShelf(shelf.id)" class="flex items-center text-sm opacity-50 hover:opacity-80 transition-opacity">
              See all <ChevronRightIcon class="size-4 ml-0.5" />
            </button>
          </div>
        </div>

        <!-- Book tile row -->
        <div class="flex gap-2 overflow-x-auto pb-1" style="-ms-overflow-style:none;scrollbar-width:none;">
          <template v-if="(shelfBooks[shelf.id] || []).length > 0">
            <div
              v-for="book in (shelfBooks[shelf.id] || []).slice(0, 4)"
              :key="book.id"
              class="flex-none w-20 h-28 rounded-xl flex items-end p-2 cursor-pointer"
              :style="{ backgroundColor: getBookColor(book.id) }"
              @click="goToBook(book.id)"
            >
              <span class="text-xs text-white font-medium leading-tight" style="display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden;">{{ book.title }}</span>
            </div>
            <div
              v-if="(shelfBooks[shelf.id] || []).length > 4"
              @click="goToShelf(shelf.id)"
              class="flex-none w-20 h-28 rounded-xl bg-base-200 flex items-center justify-center cursor-pointer"
            >
              <span class="font-bold text-lg opacity-70">+{{ (shelfBooks[shelf.id] || []).length - 4 }}</span>
            </div>
          </template>
          <div
            v-else
            @click="goToShelf(shelf.id)"
            class="flex-none w-20 h-28 rounded-xl bg-base-200 border-2 border-dashed border-base-content/20 flex items-center justify-center cursor-pointer"
          >
            <span class="text-xs opacity-30 text-center px-1">Empty</span>
          </div>
        </div>
      </div>

      <div v-if="!shelves.length" class="text-center opacity-50 py-10">No shelves yet.</div>
    </div>

    <!-- Toast -->
    <div v-if="toastMessage" class="toast toast-top toast-center pt-16">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { MinusIcon, ChevronRightIcon, UserIcon } from '@heroicons/vue/16/solid';
import CreateShelfModal from '@/components/CreateShelfModal.vue';
import { apiFetch } from '@/api/client';

const BOOK_COLORS = [
  '#1e3a5f',
  '#4a2500',
  '#2d1454',
  '#3a0f0f',
  '#0f3d2a',
  '#1a1a4e',
  '#3d2b00',
  '#1f3a2a',
  '#3a1a00',
  '#0f2a3d',
];

function getBookColor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = ((hash << 5) - hash) + id.charCodeAt(i);
    hash |= 0;
  }
  return BOOK_COLORS[Math.abs(hash) % BOOK_COLORS.length];
}

export default defineComponent({
  components: { CreateShelfModal, MinusIcon, ChevronRightIcon, UserIcon },
  setup() {
    const shelves = ref<Array<{ id: string; name: string; description: string }>>([]);
    const shelfBooks = ref<Record<string, Array<{ id: string; title: string; author: string }>>>({});
    const loading = ref(true);
    const router = useRouter();
    const toastMessage = ref('');
    const toastType = ref('');

    const totalBooks = computed(() =>
      Object.values(shelfBooks.value).reduce((sum, books) => sum + books.length, 0)
    );

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => { toastMessage.value = ''; toastType.value = ''; }, 3000);
    };

    const fetchData = async () => {
      loading.value = true;
      try {
        const res = await apiFetch('/api/shelves', { method: 'POST' });
        if (!res.ok) return;
        const data = await res.json();
        shelves.value = data.shelves;

        const bookResults = await Promise.all(
          data.shelves.map((shelf: { id: string }) =>
            apiFetch('/api/shelves/books', {
              method: 'POST',
              body: JSON.stringify({ shelf_id: shelf.id }),
            }).then(r => r.ok ? r.json() : { books: [] })
          )
        );

        const map: Record<string, Array<{ id: string; title: string; author: string }>> = {};
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
      router.push({ name: 'shelf-detail', params: { id: shelfId } });
    };

    const goToBook = (bookId: string) => {
      router.push({ name: 'book-detail', params: { id: bookId } });
    };

    const removeShelf = async (shelfId: string) => {
      try {
        const res = await apiFetch('/api/shelves/remove', {
          method: 'POST',
          body: JSON.stringify({ shelf_id: shelfId }),
        });
        if (res.ok) {
          shelves.value = shelves.value.filter(s => s.id !== shelfId);
          const map = { ...shelfBooks.value };
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

    return { shelves, shelfBooks, loading, totalBooks, fetchData, goToShelf, goToBook, removeShelf, getBookColor, toastMessage, toastType };
  },
});
</script>
