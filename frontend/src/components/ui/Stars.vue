<!-- Star rating display (gold is reserved for ratings). -->
<template>
  <span class="stars" :style="size ? { fontSize: size + 'px' } : undefined" :aria-label="`${rating} out of 5 stars`">
    <span v-for="i in 5" :key="i" :class="{ off: i > rounded }">★</span>
  </span>
</template>

<script setup lang="ts">
import {computed} from 'vue';

const props = withDefaults(
    defineProps<{
      rating: number; // 0–5, fractional ok
      size?: number; // px font-size override
    }>(),
    {size: 0},
);
const rounded = computed(() => Math.round(props.rating));
</script>

<style scoped>
.stars {
  display: inline-flex;
  gap: 2px;
  color: #d7b052;
  font-size: 13px;
  letter-spacing: 1px;
}

.stars .off {
  color: #6a6353;
}
</style>
