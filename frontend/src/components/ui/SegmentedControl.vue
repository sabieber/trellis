<!-- Segmented display-mode control.
     Usage:
       <SegmentedControl v-model="mode" :options="[{ value: 'grid' }, { value: 'list' }]">
         <template #option="{ option }"><GridIcon v-if="option.value==='grid'" />…</template>
       </SegmentedControl>
     Or simple text options: :options="[{ value: 'grid', label: 'Grid' }]" -->
<template>
  <div class="segmented-control" role="tablist">
    <button
        v-for="opt in options"
        :key="opt.value"
        role="tab"
        :aria-selected="opt.value === modelValue"
        :class="{ 'is-active': opt.value === modelValue }"
        @click="$emit('update:modelValue', opt.value)"
    >
      <slot name="option" :option="opt">{{ opt.label || opt.value }}</slot>
    </button>
  </div>
</template>

<script setup lang="ts">
export interface TSegOption {
  value: string;
  label?: string;
}

defineProps<{
  modelValue: string;
  options: TSegOption[];
}>();
defineEmits<{ (e: 'update:modelValue', value: string): void }>();
</script>

<style scoped>
.segmented-control {
  display: inline-flex;
  padding: 3px;
  gap: 2px;
  background: #221f17;
  border: 1px solid #38321f;
  border-radius: 11px;
}

.segmented-control button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 38px;
  height: 32px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  cursor: pointer;
  font-family: var(--font-sans), sans-serif;
  font-size: 12px;
  font-weight: 600;
  background: transparent;
  color: #8f866f;
  transition: 0.15s;
}

.segmented-control button.is-active {
  background: #322d1e;
  color: #ece2cc;
}
</style>
