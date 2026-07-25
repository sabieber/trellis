<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">{{ $t('readingModal.title') }}</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('readingModal.totalPages') }}</label>
          <input type="number" v-model="totalPages" class="input w-full" required />
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('readingModal.startDate') }}</label>
          <input type="date" v-model="startedAt" class="input w-full" required />
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button type="submit" class="flex-1">{{ $t('common.start') }}</Button>
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
  props: {
    initialPages: {
      type: Number,
      required: true,
    },
  },
  setup(props, { emit }) {
    const totalPages = ref(props.initialPages);
    const startedAt = ref(new Date().toISOString().split('T')[0]);

    const submitForm = () => {
      emit('submit', totalPages.value, startedAt.value);
    };

    return { totalPages, startedAt, submitForm };
  },
});
</script>
