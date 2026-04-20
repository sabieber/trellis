<template>
  <div>
    <button class="btn btn-sm btn-neutral rounded-full gap-1" @click="show = true">
      <PlusIcon class="size-4"/>
      New Shelf
    </button>
    <div v-if="show" class="modal modal-open">
      <div class="modal-box flex flex-col gap-4">
        <h3 class="font-bold text-lg">New Shelf</h3>
        <div role="alert" class="alert alert-error" v-show="errorMessage">
          <ExclamationTriangleIcon class="size-5 shrink-0"/>
          <span v-text="errorMessage"></span>
        </div>
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Name</label>
          <input type="text" v-model="name" placeholder="e.g. Want to Read" class="input input-bordered w-full" required/>
        </fieldset>
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Description <span class="opacity-50">(optional)</span></label>
          <input type="text" v-model="description" placeholder="A short description" class="input input-bordered w-full"/>
        </fieldset>
        <div class="modal-action mt-0">
          <button class="btn btn-primary flex-1" @click="createShelf" :disabled="!name">Create</button>
          <button class="btn btn-ghost" @click="cancel">Cancel</button>
        </div>
      </div>
      <div class="modal-backdrop" @click="cancel"></div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { ExclamationTriangleIcon, PlusIcon } from "@heroicons/vue/16/solid";
import { apiFetch } from '@/api/client';

export default defineComponent({
  components: { ExclamationTriangleIcon, PlusIcon },
  setup(_, { emit }) {
    const show = ref(false);
    const name = ref('');
    const description = ref('');
    const errorMessage = ref('');

    const createShelf = async () => {
      try {
        const response = await apiFetch('/api/shelves/create', {
          method: 'POST',
          body: JSON.stringify({ name: name.value, description: description.value }),
        });
        if (response.ok) {
          emit('shelfCreated');
          name.value = '';
          description.value = '';
          show.value = false;
          errorMessage.value = '';
        } else {
          const data = await response.json();
          errorMessage.value = data.error;
        }
      } catch (error) {
        errorMessage.value = 'Failed to connect to the server!';
      }
    };

    const cancel = () => {
      name.value = '';
      description.value = '';
      show.value = false;
      errorMessage.value = '';
    };

    return { show, name, description, errorMessage, createShelf, cancel };
  },
});
</script>
