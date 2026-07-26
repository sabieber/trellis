<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">{{ $t('addToShelf.title') }}</h3>
      <div v-if="loadingShelves" class="flex justify-center py-4">
        <span class="loading loading-spinner loading-md"></span>
      </div>
      <ul v-else-if="shelves.length" class="flex flex-col gap-1 max-h-72 overflow-y-auto">
        <li
          v-for="shelf in shelves"
          :key="shelf.id"
          @click="toggle(shelf.id)"
          class="flex items-center gap-3 px-3 py-2 rounded-sm cursor-pointer hover:bg-surface-2 transition-colors duration-150"
        >
          <input
            type="checkbox"
            class="checkbox checkbox-sm"
            :checked="selected.includes(shelf.id)"
            @click.stop="toggle(shelf.id)"
          />
          <span class="text-sm font-medium text-ink">{{ shelf.name || shelf.code }}</span>
          <span v-if="shelf.description" class="t-meta truncate">{{ shelf.description }}</span>
        </li>
      </ul>
      <div v-else class="t-meta text-center py-4">{{ $t('addToShelf.noShelves') }}</div>
      <div class="modal-action mt-0 flex gap-2">
        <Button variant="ghost" block @click="$emit('close')">{{ $t('common.cancel') }}</Button>
        <Button block :disabled="!selected.length || submitting" @click="confirm">
          {{ $t('addToShelf.confirm') }}
        </Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import type { PropType } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from '@/components/ui/Button.vue';
import { apiFetch } from '@/api/client';
import type { BookSearchResult } from '@/types/book';

export default defineComponent({
  components: { Button },
  props: {
    book: {
      type: Object as PropType<BookSearchResult>,
      required: true,
    },
  },
  emits: ['close', 'toast'],
  setup(props, { emit }) {
    const { t } = useI18n();
    const shelves = ref<Array<{ id: string, code: string, name: string | null, description: string }>>([]);
    const loadingShelves = ref(false);
    const selected = ref<string[]>([]);
    const submitting = ref(false);

    const toggle = (id: string) => {
      selected.value = selected.value.includes(id)
        ? selected.value.filter((x) => x !== id)
        : [...selected.value, id];
    };

    const fetchShelves = async () => {
      loadingShelves.value = true;
      try {
        const response = await apiFetch('/api/shelves', { method: 'POST' });
        if (response.ok) {
          const data = await response.json();
          shelves.value = data.shelves;
        } else {
          console.error('Failed to fetch shelves:', await response.json());
        }
      } catch (error) {
        console.error('Failed to fetch shelves:', error);
      } finally {
        loadingShelves.value = false;
      }
    };

    const addToShelf = async (shelfId: string): Promise<boolean> => {
      try {
        const response = await apiFetch('/api/shelves/add-book', {
          method: 'POST',
          body: JSON.stringify({
            shelf_id: shelfId,
            title: props.book.title,
            author: props.book.authors?.join(', '),
            isbn13: props.book.isbn13,
            isbn10: props.book.isbn10,
            google_books_id: props.book.source === 'google' ? props.book.source_id : null,
            open_library_id: props.book.source === 'openlibrary' ? props.book.source_id : null,
            cover_url: props.book.cover_url,
            page_count: props.book.page_count,
          }),
        });
        return response.ok;
      } catch (error) {
        console.error('Failed to add book to shelf:', error);
        return false;
      }
    };

    const confirm = async () => {
      if (!selected.value.length || submitting.value) return;
      submitting.value = true;
      const results = await Promise.all(selected.value.map(addToShelf));
      submitting.value = false;
      const ok = results.every(Boolean);
      emit('toast', {
        message: ok ? t('addToShelf.added') : t('addToShelf.addFailed'),
        type: ok ? 'alert-success' : 'alert-error',
      });
      emit('close');
    };

    onMounted(fetchShelves);

    return { shelves, loadingShelves, selected, submitting, toggle, confirm };
  },
});
</script>
