<!-- The list / grid / shelf / pile switch that sits in a view's title bar.
     Pair it with `BookLayout`, which renders the mode it selects. -->
<template>
  <SegmentedControl
      :model-value="modelValue"
      :options="LAYOUT_OPTIONS"
      @update:model-value="(value: string) => $emit('update:modelValue', value as LayoutMode)"
  >
    <template #option="{ option }">
      <component :is="ICONS[option.value as LayoutMode]" class="size-4"/>
    </template>
  </SegmentedControl>
</template>

<script setup lang="ts">
import type {Component} from 'vue';
import {ListIcon, LayoutGridIcon, ShelvingUnitIcon, LayersIcon} from '@lucide/vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import {LAYOUT_OPTIONS, type LayoutMode} from '@/composables/useLayoutMode';

// Typed by LayoutMode, so adding a mode fails to compile until it has a glyph —
// a `v-else` fallback would have drawn the wrong icon instead.
const ICONS: Record<LayoutMode, Component> = {
  list: ListIcon,
  grid: LayoutGridIcon,
  shelf: ShelvingUnitIcon,
  pile: LayersIcon,
};

defineProps<{ modelValue: LayoutMode }>();
defineEmits<{ 'update:modelValue': [LayoutMode] }>();
</script>
