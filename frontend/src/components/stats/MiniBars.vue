<!-- A small fixed-height categorical bar chart: one bar per bucket, value above,
     label below. Used for distributions with a handful of buckets (ratings,
     weekdays) where the full axis of the reading volume chart would be overkill. -->
<template>
  <div>
    <div class="flex items-end gap-2" :style="{height: `${trackPx}px`}">
      <div
          v-for="(bar, index) in bars"
          :key="index"
          class="flex-1 flex flex-col items-center justify-end gap-1"
      >
        <span class="stat-mono h-3">{{ bar.value ? bar.value.toLocaleString() : '' }}</span>
        <div
            class="w-full max-w-9 rounded-t bg-green transition-[height] duration-200"
            :class="{'bg-green-soft': index === peakIndex}"
            :style="{height: `${barHeight(bar.value)}px`}"
            v-on="tooltip.marks(() => `${bar.label}: ${bar.value.toLocaleString()}`)"
        ></div>
      </div>
    </div>
    <div class="flex gap-2 mt-1.5">
      <span v-for="(bar, index) in bars" :key="index" class="flex-1 text-center t-meta">{{ bar.label }}</span>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, type PropType} from 'vue';
import {useChartTooltip} from '@/composables/useChartTooltip';

export interface MiniBar {
  label: string;
  value: number;
}

/** Total track height in px; a full bar reserves 16px for the value line above it. */
const TRACK_PX = 150;
const BAR_TRACK = TRACK_PX - 16;

export default defineComponent({
  props: {
    bars: {
      type: Array as PropType<MiniBar[]>,
      required: true,
    },
    /** Highlights the tallest bar in the soft accent when true. */
    highlightPeak: {
      type: Boolean,
      default: false,
    },
  },
  setup(props) {
    const max = computed(() => Math.max(1, ...props.bars.map((bar) => bar.value)));

    const peakIndex = computed(() =>
        props.highlightPeak ? props.bars.findIndex((bar) => bar.value === max.value) : -1,
    );

    const barHeight = (value: number) => Math.round((value / max.value) * BAR_TRACK);

    return {barHeight, peakIndex, trackPx: TRACK_PX, tooltip: useChartTooltip()};
  },
});
</script>

<style scoped>
.stat-mono {
  font-family: var(--font-mono), monospace;
  font-size: 10px;
  line-height: 1;
  color: var(--color-muted);
}
</style>
