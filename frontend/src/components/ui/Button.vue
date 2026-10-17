<!-- Primary / ghost / soft button. Pass `to` when it navigates inside the app and
     `href` when it leaves it: both render a real link, which is what gives
     right-click "open in new tab". -->
<template>
  <component
      :is="to ? RouterLink : (href ? 'a' : 'button')"
      :to="to"
      :href="href"
      :target="href ? '_blank' : undefined"
      :rel="href ? 'noopener' : undefined"
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
      href?: string; // external target; opens in a new tab
    }>(),
    {variant: 'primary', block: false, icon: false, to: undefined, href: undefined},
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
  background: var(--color-green);
  color: var(--color-on-green);
}

.button--primary:hover {
  background: var(--color-green-soft);
}

.button--ghost {
  background: transparent;
  color: var(--color-ink);
  border-color: var(--color-line);
}

.button--ghost:hover {
  background: var(--color-surface-2);
  border-color: var(--color-line-hair);
}

.button--soft {
  background: color-mix(in srgb, var(--color-green) 13%, transparent);
  color: var(--color-green-soft);
  border-color: color-mix(in srgb, var(--color-green) 32%, transparent);
}

.button--soft:hover {
  background: color-mix(in srgb, var(--color-green) 20%, transparent);
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
