<!-- Donut of the readings by outcome in the period: finished, still reading and
     abandoned. The three buckets are disjoint, so they sum to every reading the
     period touched. -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t('stats.outcomesTitle') }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
      <span v-if="!loading && total > 0" class="t-meta">{{ $t('stats.readingsCount', { count: total }) }}</span>
    </div>

    <div ref="cardEl" class="bg-surface border border-line rounded-md p-4 flex-1 flex flex-col justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="total === 0" class="t-meta text-center py-14">
        {{ $t('stats.noReadings') }}
      </div>

      <div v-else class="flex flex-wrap items-center justify-center gap-x-8 gap-y-5">
        <svg viewBox="0 0 36 36" class="flex-none" :style="{width: `${donutSize}px`, height: `${donutSize}px`}">
          <circle cx="18" cy="18" r="15.915" fill="none" stroke="var(--color-surface-3)" stroke-width="3.4"/>
          <circle
              v-for="seg in arcs"
              :key="seg.key"
              cx="18"
              cy="18"
              r="15.915"
              fill="none"
              :stroke="seg.color"
              stroke-width="3.4"
              stroke-linecap="butt"
              pathLength="100"
              :stroke-dasharray="`${seg.pct} 100`"
              :stroke-dashoffset="-seg.offset"
              transform="rotate(-90 18 18)"
              class="donut-arc"
              v-on="tooltip.marks(() => seg.content)"
          />
          <text x="18" y="17.4" text-anchor="middle" class="donut-value">{{ total }}</text>
          <text x="18" y="22" text-anchor="middle" class="donut-label">{{ $t('stats.readingsLabel') }}</text>
        </svg>

        <ul class="space-y-2 min-w-40">
          <li v-for="seg in segments" :key="seg.key" class="flex items-center gap-2.5">
            <span class="size-2.5 rounded-full flex-none" :style="{background: seg.color}"></span>
            <span class="t-title text-sm flex-1">{{ seg.label }}</span>
            <span class="stat-mono">{{ seg.value }}</span>
            <span class="t-meta w-10 text-right">{{ seg.share }}%</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, onMounted, ref, type PropType} from 'vue';
import {useI18n} from 'vue-i18n';
import type {ReadingStates} from '@/composables/useStatsBreakdown';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {useChartTooltip} from '@/composables/useChartTooltip';
import {formatPeriod} from '@/utils/period';

// Chart-local literals, like the heatmap ramp: not part of the shared token set.
const COLORS = {finished: '#93c456', reading: '#c8a13c', abandoned: '#6b5f4a'};

export default defineComponent({
  props: {
    mode: {type: String, required: true},
    year: {type: Number, required: true},
    month: {type: Number, required: true},
    states: {type: Object as PropType<ReadingStates | null>, default: null},
    loading: {type: Boolean, default: false},
  },
  setup(props) {
    const {t} = useI18n();
    const cardEl = ref<HTMLElement | null>(null);
    const {containerWidth, setupObserver} = useContainerWidth(cardEl);
    onMounted(setupObserver);

    // Scale the donut with the card width so it fills the (often stretched) card
    // instead of floating small. Legend needs ~160px beside it. Sizing off width
    // keeps it stable — the donut height can't feed back into the measurement.
    const donutSize = computed(() => {
      // Reserve ~160px for the legend and ~40px for the gap beside the donut.
      // Capped at 190 so the card matches the five-row top-authors card beside it.
      const beside = (containerWidth.value || 420) - 200;
      return Math.round(Math.max(150, Math.min(beside, 190)));
    });

    const total = computed(() => {
      const s = props.states;
      return s ? s.finished + s.reading + s.abandoned : 0;
    });

    const segments = computed(() => {
      const s = props.states ?? {finished: 0, reading: 0, abandoned: 0};
      return [
        {key: 'finished', label: t('bookDetail.stateFinished'), value: s.finished, color: COLORS.finished},
        {key: 'reading', label: t('bookDetail.stateReading'), value: s.reading, color: COLORS.reading},
        {key: 'abandoned', label: t('bookDetail.stateAbandoned'), value: s.abandoned, color: COLORS.abandoned},
      ].map((seg) => ({
        ...seg,
        share: total.value ? Math.round((seg.value / total.value) * 100) : 0,
      }));
    });

    // Only non-empty segments get an arc, each offset by the ones before it.
    const arcs = computed(() => {
      let offset = 0;
      return segments.value
          .filter((seg) => seg.value > 0)
          .map((seg) => {
            const pct = (seg.value / total.value) * 100;
            const content = `${seg.label}: ${seg.value} (${seg.share}%)`;
            const arc = {key: seg.key, color: seg.color, pct, offset, content};
            offset += pct;
            return arc;
          });
    });

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    return {cardEl, donutSize, total, segments, arcs, periodLabel, tooltip: useChartTooltip()};
  },
});
</script>

<style scoped>
.donut-arc {
  cursor: pointer;
}

.donut-value {
  font-family: var(--font-mono), monospace;
  font-size: 8px;
  font-weight: 600;
  fill: var(--color-ink);
}

.donut-label {
  font-size: 2.6px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  fill: var(--color-muted);
}

.stat-mono {
  font-family: var(--font-mono), monospace;
  font-size: 12px;
  color: var(--color-ink-2);
}
</style>
