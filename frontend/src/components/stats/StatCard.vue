<template>
  <div class="bg-surface border border-line rounded-md p-4">
    <div class="flex items-center gap-2 mb-2.5">
      <component :is="icon" class="size-4.5 text-green-soft flex-none"/>
      <span class="t-eyebrow truncate">{{ label }}</span>
    </div>
    <div class="t-display text-[26px]">{{ value }}</div>
    <div v-if="subtext" class="t-meta mt-1">{{ subtext }}</div>
  </div>
</template>

<script lang="ts">
import {defineComponent} from 'vue';
import type {Component, PropType} from 'vue';

export default defineComponent({
  props: {
    icon: {
      type: [Object, Function] as PropType<Component>,
      required: true,
    },
    label: {
      type: String,
      required: true,
    },
    value: {
      type: [String, Number],
      required: true,
    },
    subtext: {
      type: String,
      default: '',
    },
  },
});
</script>
