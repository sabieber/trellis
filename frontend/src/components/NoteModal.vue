<!-- Writes a note and edits one: the fields are the same, so `note` decides
     which of the two it is. -->
<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">{{ note ? $t('noteModal.editTitle') : $t('noteModal.newTitle') }}</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('noteModal.text') }}</label>
          <textarea v-model="text" class="textarea w-full" rows="6" required autofocus></textarea>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('noteModal.page') }} ({{ $t('common.optional') }})</label>
          <input type="number" v-model="page" class="input w-full" min="1"/>
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button type="submit" class="flex-1">{{ $t('common.save') }}</Button>
          <Button variant="ghost" type="button" @click="$emit('close')">{{ $t('common.cancel') }}</Button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, type PropType} from 'vue';
import Button from '@/components/ui/Button.vue';
import type {BookNote} from '@/types/book';

export default defineComponent({
  components: {Button},
  props: {
    note: {type: Object as PropType<BookNote | null>, default: null},
  },
  emits: ['save', 'close'],
  setup(props, {emit}) {
    const text = ref(props.note?.text ?? '');
    // An empty field yields '' from v-model, which is "no page", not 0.
    const page = ref<number | string>(props.note?.page ?? '');

    const submitForm = () => {
      const pageNumber = parseInt(String(page.value), 10);
      emit('save', text.value, Number.isNaN(pageNumber) ? null : pageNumber);
    };

    return {text, page, submitForm};
  },
});
</script>
