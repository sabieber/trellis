<!-- Primary / ghost / soft button. Pass `to` when it navigates: it then renders
     a real link, which is what gives right-click "open in new tab". -->
<template>
  <component
      :is="to ? RouterLink : 'button'"
      :to="to"
      class="button"
      :class="[`button--${variant}`, { 'button--block': block, 'button--icon': icon }]"
  >
    <slot></slot>
  </component>
</template>

<script setup lang="ts">
import {RouterLink, type RouteLocationRaw} from 'vue-router';

withDefaults(
    defineProps<{
      variant?: 'primary' | 'ghost' | 'soft';
      block?: boolean;
      icon?: boolean; // square 42px icon-only button
      to?: RouteLocationRaw;
    }>(),
    {variant: 'primary', block: false, icon: false, to: undefined},
);
</script>

<style scoped>
.button {
  display: inline-flex;
  text-decoration: none;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-family: var(--font-sans), sans-serif;
  font-weight: 600;
  font-size: 14px;
  padding: 11px 18px;
  border-radius: 11px;
  border: 1px solid transparent;
  cursor: pointer;
  line-height: 1;
  transition: 0.16s ease;
}

.button--primary {
  background: #93c456;
  color: #16170d;
}

.button--primary:hover {
  background: #a7d06e;
}

.button--ghost {
  background: transparent;
  color: #ece2cc;
  border-color: #38321f;
}

.button--ghost:hover {
  background: #2a2619;
  border-color: rgb(236 226 204 / 0.12);
}

.button--soft {
  background: rgb(147 196 86 / 0.13);
  color: #a7d06e;
  border-color: rgb(147 196 86 / 0.32);
}

.button--soft:hover {
  background: rgb(147 196 86 / 0.2);
}

.button--block {
  width: 100%;
}

.button--icon {
  padding: 11px;
  width: 42px;
  height: 42px;
}

.button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
