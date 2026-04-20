<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="font-bold text-lg">Track Progress</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Page Number</label>
          <input type="number" v-model="progress" class="input input-bordered w-full" ref="progressInput" required />
        </fieldset>
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Date</label>
          <input type="date" v-model="readAt" class="input input-bordered w-full" required />
        </fieldset>
        <div class="modal-action mt-0">
          <button type="submit" class="btn btn-primary flex-1">Submit</button>
          <button type="button" @click="$emit('close')" class="btn btn-ghost">Cancel</button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, watch } from 'vue';

export default defineComponent({
  props: {
    initialProgress: {
      type: Number,
      required: true,
    },
  },
  setup(props, { emit }) {
    const progress = ref(props.initialProgress);
    const readAt = ref(new Date().toISOString().split('T')[0]);
    const progressInput = ref<HTMLInputElement | null>(null);

    const submitForm = () => {
      emit('submit', progress.value, readAt.value);
    };

    watch(() => props.initialProgress, (newVal) => {
      progress.value = newVal;
    });

    onMounted(() => {
      progressInput.value?.focus();
      progressInput.value?.select();
    });

    return { progress, readAt, submitForm, progressInput };
  },
});
</script>
