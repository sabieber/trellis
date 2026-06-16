<!-- Filter pill. Active state = green tint, never solid fill. -->
<template>
  <button class="chip" :class="{ 'is-active': active }">
    <slot></slot>
  </button>
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
  color: #b6ab8f;
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid #38321f;
  background: #221f17;
  white-space: nowrap;
  cursor: pointer;
  transition: 0.15s;
}

.chip:hover {
  border-color: rgb(236 226 204 / 0.12);
  color: #ece2cc;
}

.chip.is-active {
  background: rgb(147 196 86 / 0.13);
  border-color: rgb(147 196 86 / 0.32);
  color: #a7d06e;
}
</style>
