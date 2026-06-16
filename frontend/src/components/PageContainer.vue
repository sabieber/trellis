<template>
  <div class="min-h-screen flex flex-col items-center">
    <div class="w-full max-w-lg p-6 flex flex-col grow">
      <div class="flex justify-between items-center mb-4">
        <h2 class="t-display text-2xl">{{ title }}</h2>
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
import { defineComponent, ref } from 'vue';

export default defineComponent({
  props: {
    title: {
      type: String,
      required: true
    }
  },
  setup(_, { expose }) {
    const toastMessage = ref('');
    const toastType = ref('');

    const showToast = ({ message, type }: { message: string; type: string }) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    expose({ showToast });

    return {
      toastMessage,
      toastType,
      showToast,
    };
  }
});
</script>
