<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">Start Reading</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Total Pages</label>
          <input type="number" v-model="totalPages" class="input w-full" required />
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button type="submit" class="flex-1">Start</Button>
          <Button variant="ghost" type="button" @click="$emit('close')">Cancel</Button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import Button from '@/components/ui/Button.vue';

export default defineComponent({
  components: { Button },
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
