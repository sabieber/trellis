<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">Add to Shelf</h3>
      <div v-if="loadingShelves" class="flex justify-center py-4">
        <span class="loading loading-spinner loading-md"></span>
      </div>
      <ul v-else-if="shelves.length" class="flex flex-col gap-1">
        <li
          v-for="shelf in shelves"
          :key="shelf.id"
          @click="addBookToShelf(shelf.id)"
          class="flex items-center gap-3 px-3 py-2 rounded-sm cursor-pointer hover:bg-surface-2 transition-colors duration-150"
        >
          <span class="text-sm font-medium text-ink">{{ shelf.name }}</span>
          <span v-if="shelf.description" class="t-meta truncate">{{ shelf.description }}</span>
        </li>
      </ul>
      <div v-else class="t-meta text-center py-4">No shelves found.</div>
      <div class="modal-action mt-0">
        <Button variant="ghost" block @click="$emit('close')">Cancel</Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import type { PropType } from 'vue';
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
  setup(props, { emit }) {
    const shelves = ref<Array<{ id: string, name: string, description: string }>>([]);
    const loadingShelves = ref(false);

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

    const addBookToShelf = async (shelfId: string) => {
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
          }),
        });
        if (response.ok) {
          emit('toast', { message: 'Book added to shelf successfully.', type: 'alert-success' });
          emit('close');
        } else {
          console.error('Failed to add book to shelf:', await response.json());
          emit('toast', { message: 'Failed to add book to shelf.', type: 'alert-error' });
        }
      } catch (error) {
        console.error('Failed to add book to shelf:', error);
        emit('toast', { message: 'Failed to add book to shelf.', type: 'alert-error' });
      }
    };

    onMounted(fetchShelves);

    return { shelves, loadingShelves, addBookToShelf };
  },
});
</script>
