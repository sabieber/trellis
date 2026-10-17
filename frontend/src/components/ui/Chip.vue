<!-- Tag / label pill (non-interactive). Active state = green tint, never solid fill. -->
<template>
  <span class="chip" :class="{ 'is-active': active }">
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ active?: boolean }>(), {active: false});
</script>

<style scoped>
.chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-family: var(--font-sans), sans-serif;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-ink-dim);
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid var(--color-line);
  background: var(--color-surface);
  white-space: nowrap;
}

.chip.is-active {
  background: color-mix(in srgb, var(--color-green) 13%, transparent);
  border-color: color-mix(in srgb, var(--color-green) 32%, transparent);
  color: var(--color-green-soft);
}
</style>
