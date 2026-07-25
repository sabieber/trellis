<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">{{ $t('goalModal.title') }}</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('goalModal.type') }}</label>
          <select v-model="goalType" class="select w-full">
            <option value="books">{{ $t('common.books') }}</option>
            <option value="pages">{{ $t('common.pages') }}</option>
          </select>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('goalModal.timeframe') }}</label>
          <select v-model="timeframe" class="select w-full">
            <option value="year">{{ $t('goalModal.year') }}</option>
            <option value="month">{{ $t('goalModal.month') }}</option>
            <option value="week">{{ $t('goalModal.week') }}</option>
          </select>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ goalType === 'books' ? $t('goalModal.numberOfBooks') : $t('goalModal.numberOfPages') }}</label>
          <input type="number" v-model="target" class="input w-full" min="1" required />
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button type="submit" class="flex-1">{{ $t('common.create') }}</Button>
          <Button variant="ghost" type="button" @click="$emit('close')">{{ $t('common.cancel') }}</Button>
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
