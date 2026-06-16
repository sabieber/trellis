<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">Create Goal</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Type</label>
          <select v-model="goalType" class="select w-full">
            <option value="books">Books</option>
            <option value="pages">Pages</option>
          </select>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">Timeframe</label>
          <select v-model="timeframe" class="select w-full">
            <option value="year">Year</option>
            <option value="month">Month</option>
            <option value="week">Week</option>
          </select>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ goalType === 'books' ? 'Number of Books' : 'Number of Pages' }}</label>
          <input type="number" v-model="target" class="input w-full" min="1" required />
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button type="submit" class="flex-1">Create</Button>
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
  setup(_props, { emit }) {
    const goalType = ref('books');
    const timeframe = ref('year');
    const target = ref(1);

    const submitForm = () => {
      emit('submit', {
        goalType: goalType.value,
        timeframe: timeframe.value,
        target: target.value,
      });
    };

    return { goalType, timeframe, target, submitForm };
  },
});
</script>
