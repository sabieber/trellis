<template>
  <div class="modal modal-open">
    <div class="modal-box flex flex-col gap-4">
      <h3 class="t-title text-lg">{{ $t('readingModal.title') }}</h3>
      <form @submit.prevent.stop="submitForm" class="contents">
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('readingModal.trackingMode') }}</label>
          <SegmentedControl v-model="mode" :options="modeOptions" />
        </fieldset>
        <fieldset v-if="mode === 'pages'" class="flex flex-col gap-1.5">
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
import { computed, defineComponent, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from '@/components/ui/Button.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';

export default defineComponent({
  components: { Button, SegmentedControl },
  props: {
    initialPages: {
      type: Number,
      required: true,
    },
  },
  setup(props, { emit }) {
    const { t } = useI18n();
    const mode = ref('pages');
    const totalPages = ref(props.initialPages);
    const startedAt = ref(new Date().toISOString().split('T')[0]);

    const modeOptions = computed(() => [
      { value: 'pages', label: t('readingModal.modePages') },
      { value: 'percentage', label: t('readingModal.modePercentage') },
    ]);

    const submitForm = () => {
      emit('submit', mode.value, totalPages.value, startedAt.value);
    };

    return { mode, modeOptions, totalPages, startedAt, submitForm };
  },
});
</script>
