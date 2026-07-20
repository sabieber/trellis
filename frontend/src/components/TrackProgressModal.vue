<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">Track Progress</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Date</label>
          <input type="date" v-model="readAt" class="input w-full" required />
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Page Number</label>
          <BookProgressInput v-if="totalPages > 0" v-model="progress" :total-pages="totalPages" />
          <input v-else type="number" v-model="progress" class="input w-full" ref="progressInput" required />
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button type="submit" class="flex-1">Submit</Button>
          <Button variant="ghost" type="button" @click="$emit('close')">Cancel</Button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, watch } from 'vue';
import Button from '@/components/ui/Button.vue';
import BookProgressInput from '@/components/BookProgressInput.vue';

export default defineComponent({
  components: { Button, BookProgressInput },
  props: {
    initialProgress: {
      type: Number,
      required: true,
    },
    totalPages: {
      type: Number,
      default: 0,
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
