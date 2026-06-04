<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="font-bold text-lg">Create Goal</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Type</label>
          <select v-model="goalType" class="select select-bordered w-full">
            <option value="books">Books</option>
            <option value="pages">Pages</option>
          </select>
        </fieldset>
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">Timeframe</label>
          <select v-model="timeframe" class="select select-bordered w-full">
            <option value="year">Year</option>
            <option value="month">Month</option>
            <option value="week">Week</option>
          </select>
        </fieldset>
        <fieldset class="flex flex-col gap-1">
          <label class="text-sm font-medium opacity-70">{{ goalType === 'books' ? 'Number of Books' : 'Number of Pages' }}</label>
          <input type="number" v-model="target" class="input input-bordered w-full" min="1" required />
        </fieldset>
        <div class="modal-action mt-0">
          <button type="submit" class="btn btn-primary flex-1">Create</button>
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
