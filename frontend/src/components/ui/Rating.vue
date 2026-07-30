<template>
  <span
      class="book-rating"
      :class="{ interactive }"
      :style="size ? { fontSize: size + 'px' } : undefined"
      :aria-label="$t('common.ratingAria', { rating: displayRating })"
      @mouseleave="hoverVal = 0"
  >
    <span
        v-for="i in 5"
        :key="i"
        :class="{ off: i > displayRating, hover: interactive && hoverVal >= i }"
        @mouseenter="interactive && (hoverVal = i)"
        @click="interactive && click(i)"
    >
      <FlowerIcon fill="color-mix(in srgb, currentColor 50%, transparent)"/>
    </span>
  </span>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue';
import {FlowerIcon} from "@lucide/vue";

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
.book-rating {
  display: inline-flex;
  gap: 2px;
  color: var(--color-gold);
  font-size: 13px;
}

/* Sized in em so the existing font-size / size prop keeps driving the glyph. */
.book-rating svg {
  display: block;
  width: 1em;
  height: 1em;
}

.book-rating .off {
  color: var(--color-faint);
}

.book-rating.interactive span {
  cursor: pointer;
  transition: color 0.1s, transform 0.1s;
}

.book-rating.interactive span:hover {
  transform: scale(1.15);
}

.book-rating .hover {
  color: var(--color-gold-bright);
}
</style>
