<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="font-bold text-lg">Add to Shelf</h3>
      <div v-if="loadingShelves" class="flex justify-center py-4">
        <span class="loading loading-spinner loading-md"></span>
      </div>
      <ul v-else-if="shelves.length" class="flex flex-col gap-1">
        <li
          v-for="shelf in shelves"
          :key="shelf.id"
          @click="addBookToShelf(shelf.id)"
          class="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-base-200 transition-colors"
        >
          <span class="font-medium">{{ shelf.name }}</span>
          <span v-if="shelf.description" class="text-sm opacity-50 truncate">{{ shelf.description }}</span>
        </li>
      </ul>
      <div v-else class="text-center opacity-50 py-4">No shelves found.</div>
      <div class="modal-action mt-0">
        <button @click="$emit('close')" class="btn btn-ghost w-full">Cancel</button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import type { PropType } from 'vue';
import { apiFetch } from '@/api/client';

export default defineComponent({
  props: {
    book: {
      type: Object as PropType<any>,
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
            title: props.book.volumeInfo.title,
            author: props.book.volumeInfo.authors?.join(', '),
            isbn13: props.book.volumeInfo.industryIdentifiers?.find((id: any) => id.type === 'ISBN_13')?.identifier,
            isbn10: props.book.volumeInfo.industryIdentifiers?.find((id: any) => id.type === 'ISBN_10')?.identifier,
            google_books_id: props.book.id,
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
