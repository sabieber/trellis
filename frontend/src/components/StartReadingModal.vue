<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="font-bold text-lg">Start Reading</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Total Pages</label>
          <input type="number" v-model="totalPages" class="input input-bordered w-full" required />
        </fieldset>
        <div class="modal-action mt-0">
          <button type="submit" class="btn btn-primary flex-1">Start</button>
          <button type="button" @click="$emit('close')" class="btn btn-ghost">Cancel</button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';

export default defineComponent({
  props: {
    initialPages: {
      type: Number,
      required: true,
    },
  },
  setup(props, { emit }) {
    const totalPages = ref(props.initialPages);

    const submitForm = () => {
      emit('submit', totalPages.value);
    };

    return { totalPages, submitForm };
  },
});
</script>
