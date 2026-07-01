<template>
  <span
      class="stars"
      :class="{ interactive }"
      :style="size ? { fontSize: size + 'px' } : undefined"
      :aria-label="`${displayRating} out of 5 stars`"
      @mouseleave="hoverVal = 0"
  >
    <span
        v-for="i in 5"
        :key="i"
        :class="{ off: i > displayRating, hover: interactive && hoverVal >= i }"
        @mouseenter="interactive && (hoverVal = i)"
        @click="interactive && click(i)"
    >★</span>
  </span>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue';

const props = withDefaults(
    defineProps<{
      rating: number;
      size?: number;
      interactive?: boolean;
    }>(),
    {size: 0, interactive: false},
);

const emit = defineEmits<{
  update: [value: number | null];
}>();

const hoverVal = ref(0);

const displayRating = computed(() => {
  if (props.interactive && hoverVal.value > 0) return hoverVal.value;
  return Math.round(props.rating);
});

const click = (val: number) => {
  emit('update', val === props.rating ? null : val);
};
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

.stars.interactive span {
  cursor: pointer;
  transition: color 0.1s, transform 0.1s;
}

.stars.interactive span:hover {
  transform: scale(1.15);
}

.stars .hover {
  color: #e8c462;
}
</style>
