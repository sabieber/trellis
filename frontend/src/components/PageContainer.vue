<template>
  <div :class="['min-h-screen flex flex-col', { 'items-center': !wide }]">
    <div :class="['w-full flex flex-col grow', wide ? 'px-4 pt-5 pb-4' : 'max-w-lg p-6']">
      <div class="flex justify-between items-start mb-4 gap-4">
        <div class="min-w-0">
          <h2 class="t-display text-2xl truncate">{{ title }}</h2>
          <p v-if="description" class="t-meta mt-1">{{ description }}</p>
        </div>
        <slot name="title-button"></slot>
      </div>
      <slot></slot>
    </div>
    <div v-if="toastMessage" class="toast toast-top toast-center pt-16">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref} from 'vue';

export default defineComponent({
  props: {
    title: {
      type: String,
      required: true
    },
    description: {
      type: String,
      default: ''
    },
    wide: {
      type: Boolean,
      default: false
    }
  },
  setup(_, {expose}) {
    const toastMessage = ref('');
    const toastType = ref('');

    const showToast = ({message, type}: { message: string; type: string }) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    expose({showToast});

    return {
      toastMessage,
      toastType,
      showToast,
    };
  }
});
</script>
