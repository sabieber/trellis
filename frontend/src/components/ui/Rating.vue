<template>
  <!-- Thumbs mode: one thumb per direction plus the sideways one in the middle,
       which is both the readout and the way to set a middling score. The lit
       thumb is the tendency of the stored score. -->
  <span
      v-if="thumbs"
      class="book-rating"
      :class="{ interactive }"
      :style="size ? { fontSize: size + 'px' } : undefined"
      :aria-label="thumbsAria"
      @mouseleave="hoverSide = null"
  >
    <span
        v-for="side in sides"
        :key="side"
        :class="{ off: shown !== side, neutral: side === 0 }"
        @mouseenter="interactive && (hoverSide = side)"
        @click="interactive && pick(side)"
    >
      <component :is="side === -1 ? ThumbsDownIcon : ThumbsUpIcon"/>
    </span>
  </span>

  <span
      v-else
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
import {useI18n} from 'vue-i18n';
import {FlowerIcon, ThumbsDownIcon, ThumbsUpIcon} from "@lucide/vue";
import {ratingMode, tendency, THUMBS_DOWN, THUMBS_MIDDLE, THUMBS_UP} from '@/utils/ratingMode';

const props = withDefaults(
    defineProps<{
      rating: number;
      size?: number;
      interactive?: boolean;
      /** Forces the star scale, for a rating that is not the user's own. */
      stars?: boolean;
    }>(),
    {size: 0, interactive: false, stars: false},
);

const emit = defineEmits<{
  update: [value: number | null];
}>();

const {t} = useI18n();

/** The three thumbs, in the order they are drawn. `0` is the sideways one. */
type Side = -1 | 0 | 1;
const SIDES: Side[] = [-1, 0, 1];

const hoverVal = ref(0);
const hoverSide = ref<Side | null>(null);

const thumbs = computed(() => !props.stars && ratingMode.value === 'thumbs');

const displayRating = computed(() => {
  if (props.interactive && hoverVal.value > 0) return hoverVal.value;
  return Math.round(props.rating);
});

/**
 * The thumb that is lit right now: the hovered one wins over the stored one.
 * `null` means nothing is lit, which is what an unrated book looks like.
 */
const shown = computed<Side | null>(() => {
  if (props.interactive && hoverSide.value !== null) return hoverSide.value;
  return props.rating ? tendency(props.rating) : null;
});

/**
 * All three thumbs while rating; only the lit one as a readout. The full set
 * never changes on hover — a row that reflows under the pointer is unclickable.
 */
const sides = computed(() => {
  if (props.interactive) return SIDES;
  return shown.value === null ? [] : [shown.value];
});

const thumbsAria = computed(() => {
  if (shown.value === 1) return t('common.thumbUpAria');
  if (shown.value === -1) return t('common.thumbDownAria');
  if (shown.value === 0) return t('common.thumbNeutralAria');
  return t('common.notRated');
});

const click = (val: number) => {
  emit('update', val === props.rating ? null : val);
};

/**
 * A thumb only writes when it changes the tendency. Clicking the lit thumb
 * removes the rating, exactly like clicking the lit star — it never rewrites a
 * 4 into a 5 behind the user's back.
 */
const pick = (side: Side) => {
  if (props.rating && tendency(props.rating) === side) {
    emit('update', null);
    return;
  }
  emit('update', side === 1 ? THUMBS_UP : side === -1 ? THUMBS_DOWN : THUMBS_MIDDLE);
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

/* The middling score: a thumb turned on its side. */
.book-rating .neutral svg {
  transform: rotate(-90deg);
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
