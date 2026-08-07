<template>
  <PageContainer :title="shelf.name || shelf.code" :description="shelf.description" wide ref="pageContainer">
    <template #title>
      <h2 class="t-display text-2xl truncate">
        <InlineEdit
            :value="shelf.name"
            :placeholder="shelf.code"
            :save="saveName"
            :label="$t('shelf.editName')"
        />
      </h2>
      <p v-if="shelf.name" class="t-mono mt-0.5">{{ shelf.code }}</p>
      <p v-if="shelf.description" class="t-meta mt-1">{{ shelf.description }}</p>
    </template>

    <template #title-button>
      <div v-if="!loading && books.length" class="flex items-center gap-2">
        <Button variant="ghost" icon :title="$t('shelf.random')" :aria-label="$t('shelf.random')"
                @click="pickerOpen = true">
          <Dice3Icon/>
        </Button>
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="added_at">{{ $t('shelf.sortAdded') }}</option>
          <option value="title">{{ $t('shelf.sortTitle') }}</option>
          <option value="author">{{ $t('shelf.sortAuthor') }}</option>
        </select>
        <LayoutModeSelect v-model="layoutMode"/>
      </div>
    </template>

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <BookLayout
        v-else-if="books.length"
        :books="sortedBooks"
        :mode="layoutMode"
        removable
        @view-book="viewBookDetail"
        @remove-book="confirmRemoveBook"
        @view-author="viewAuthor"
    />

    <div v-else class="t-meta text-center py-12">{{ $t('shelf.noBooks') }}</div>

    <RandomBookModal
        v-if="pickerOpen"
        :books="books"
        @close="pickerOpen = false"
        @view-book="viewBookDetail"
    />

    <ConfirmDialog
        v-if="pendingRemoveBookId"
        :title="$t('shelf.removeBookTitle')"
        :message="$t('shelf.removeBookMessage')"
        :confirmLabel="$t('common.remove')"
        @confirm="removeBookFromShelf"
        @cancel="pendingRemoveBookId = null"
    />
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {useI18n} from 'vue-i18n';
import {Dice3Icon} from "@lucide/vue";
import PageContainer from '@/components/PageContainer.vue';
import InlineEdit from '@/components/ui/InlineEdit.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import RandomBookModal from '@/components/shelf/RandomBookModal.vue';
import Button from '@/components/ui/Button.vue';
import {apiFetch} from '@/api/client';
import {apiErrorMessage} from '@/utils/apiError';
import {goToAuthor} from '@/utils/authorRoute';
import {useLayoutMode} from '@/composables/useLayoutMode';
import type {ShelfBook} from '@/types/shelf';

export default defineComponent({
  components: {
    Dice3Icon,
    PageContainer, ConfirmDialog, InlineEdit,
    BookLayout, LayoutModeSelect,
    RandomBookModal, Button,
  },
  setup() {
    const {t} = useI18n();
    const route = useRoute();
    const router = useRouter();
    const books = ref<ShelfBook[]>([]);
    const loading = ref(true);
    const shelf = ref<{ code: string; name: string | null; description: string }>({
      code: '',
      name: null,
      description: ''
    });
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');
    const pageContainer = ref<any>(null);
    const pendingRemoveBookId = ref<string | null>(null);
    const pickerOpen = ref(false);
    const layoutMode = useLayoutMode();

    const sortedBooks = computed(() => {
      const arr = [...books.value];
      if (sortBy.value === 'title') {
        arr.sort((a, b) => a.title.localeCompare(b.title));
      } else if (sortBy.value === 'author') {
        arr.sort((a, b) => a.author.localeCompare(b.author));
      } else {
        arr.sort((a, b) => b.added_at.localeCompare(a.added_at));
      }
      return arr;
    });

    const fetchShelfBooks = async (shelfId: string) => {
      try {
        const response = await apiFetch('/api/shelves/books', {
          method: 'POST',
          body: JSON.stringify({shelf_id: shelfId}),
        });
        if (response.ok) {
          const data = await response.json();
          books.value = data.books;
          shelf.value = data.shelf;
        } else {
          console.error('Failed to fetch books:', await response.json());
        }
      } catch (error) {
        console.error('Failed to fetch books:', error);
      } finally {
        loading.value = false;
      }
    };

    const confirmRemoveBook = (bookId: string) => {
      pendingRemoveBookId.value = bookId;
    };

    const removeBookFromShelf = async () => {
      const bookId = pendingRemoveBookId.value;
      if (!bookId) return;
      pendingRemoveBookId.value = null;
      try {
        const response = await apiFetch('/api/shelves/remove-book', {
          method: 'POST',
          body: JSON.stringify({book_id: bookId, shelf_id: route.params.id}),
        });
        if (response.ok) {
          pageContainer.value?.showToast({message: t('shelf.bookRemoved'), type: 'alert-success'});
          books.value = books.value.filter((book) => book.id !== bookId);
        } else {
          console.error('Failed to remove book:', await response.json());
          pageContainer.value?.showToast({message: apiErrorMessage(response.status, t), type: 'alert-error'});
        }
      } catch (error) {
        console.error('Failed to remove book:', error);
        pageContainer.value?.showToast({message: t('error.network'), type: 'alert-error'});
      }
    };

    const saveName = async (value: string): Promise<boolean> => {
      const name = value.trim() || null;
      try {
        const response = await apiFetch('/api/shelves/set-name', {
          method: 'POST',
          body: JSON.stringify({shelf_id: route.params.id, name}),
        });
        if (response.ok) {
          shelf.value.name = name;
          return true;
        }
        pageContainer.value?.showToast({message: apiErrorMessage(response.status, t), type: 'alert-error'});
      } catch {
        pageContainer.value?.showToast({message: t('error.network'), type: 'alert-error'});
      }
      return false;
    };

    const viewBookDetail = (id: string) => {
      router.push({name: 'book-detail', params: {id}});
    };

    const viewAuthor = (author: string) => goToAuthor(router, author);

    onMounted(() => fetchShelfBooks(route.params.id as string));

    return {
      books, sortedBooks, loading, shelf, sortBy, layoutMode,
      pageContainer, pendingRemoveBookId, pickerOpen,
      confirmRemoveBook, removeBookFromShelf, viewBookDetail, viewAuthor, saveName,
    };
  },
});
</script>
