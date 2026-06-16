<template>
  <div>
    <Button variant="soft" class="px-3.5! py-2! text-[13px]!" @click="show = true">
      <PlusIcon class="size-4" />
      New Shelf
    </Button>
    <div v-if="show" class="modal modal-open">
      <div class="modal-box flex flex-col gap-4">
        <h3 class="t-title text-lg">New Shelf</h3>
        <div role="alert" class="alert alert-error" v-show="errorMessage">
          <ExclamationTriangleIcon class="size-5 shrink-0"/>
          <span v-text="errorMessage"></span>
        </div>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Name</label>
          <input type="text" v-model="name" placeholder="e.g. Want to Read" class="input w-full" required/>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Description <span class="text-faint">(optional)</span></label>
          <input type="text" v-model="description" placeholder="A short description" class="input w-full"/>
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button class="flex-1" :disabled="!name" @click="createShelf">Create</Button>
          <Button variant="ghost" @click="cancel">Cancel</Button>
        </div>
      </div>
      <div class="modal-backdrop" @click="cancel"></div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { ExclamationTriangleIcon, PlusIcon } from "@heroicons/vue/24/outline";
import Button from '@/components/ui/Button.vue';
import { apiFetch } from '@/api/client';

export default defineComponent({
  components: { ExclamationTriangleIcon, PlusIcon, Button },
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
