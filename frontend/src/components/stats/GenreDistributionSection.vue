<!-- Pie of the genres of the books finished in the period. A book counts in each
     of its genres, so the slices are shares of the genre labels, not of the
     books. Only the seven biggest genres get a slice; the rest collapse into
     "Other" so the pie stays readable. -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t('stats.genresTitle') }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
      <span v-if="!loading && total > 0" class="t-meta">{{ $t('stats.genresCount', { count: distribution.length }) }}</span>
    </div>

    <div ref="cardEl" class="bg-surface border border-line rounded-md p-4 flex-1 flex flex-col justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="total === 0" class="t-meta text-center py-14">
        {{ $t('stats.noGenres') }}
      </div>

      <div v-else class="flex flex-wrap items-center justify-center gap-x-8 gap-y-5">
        <svg viewBox="0 0 36 36" class="flex-none" :style="{width: `${pieSize}px`, height: `${pieSize}px`}">
          <circle
              v-for="slice in slices"
              :key="slice.key"
              cx="18"
              cy="18"
              r="8.75"
              fill="none"
              :stroke="slice.color"
              stroke-width="17.5"
              pathLength="100"
              :stroke-dasharray="`${slice.pct} 100`"
              :stroke-dashoffset="-slice.offset"
              transform="rotate(-90 18 18)"
              class="pie-slice"
              v-on="tooltip.marks(() => slice.content)"
          />
        </svg>

        <ul class="space-y-2 min-w-40 max-w-64">
          <li v-for="slice in slices" :key="slice.key" class="flex items-center gap-2.5">
            <span class="size-2.5 rounded-full flex-none" :style="{background: slice.color}"></span>
            <span class="t-title text-sm flex-1 truncate" :title="slice.label">{{ slice.label }}</span>
            <span class="stat-mono">{{ slice.value }}</span>
            <span class="t-meta w-10 text-right">{{ slice.share }}%</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, onMounted, ref, type PropType} from 'vue';
import {useI18n} from 'vue-i18n';
import type {GenreStat} from '@/composables/useStatsBreakdown';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {useChartTooltip} from '@/composables/useChartTooltip';
import {formatPeriod} from '@/utils/period';

// Chart-local literals, like the outcome donut: not part of the shared token
// set. The order is the colourblind-safe one — never sort or cycle it.
const COLORS = ['#6f9d3c', '#b5502a', '#3f7fc4', '#c98500', '#199e70', '#8a6fd0', '#d55181'];
const OTHER_COLOR = '#6b5f4a';

export default defineComponent({
  props: {
    mode: {type: String, required: true},
    year: {type: Number, required: true},
    month: {type: Number, required: true},
    distribution: {type: Array as PropType<GenreStat[]>, default: () => []},
    loading: {type: Boolean, default: false},
  },
  setup(props) {
    const {t} = useI18n();
    const cardEl = ref<HTMLElement | null>(null);
    const {containerWidth, setupObserver} = useContainerWidth(cardEl);
    onMounted(setupObserver);

    // Same sizing rule as the outcome donut: grow with the card, but leave the
    // legend its room and stop at 190 so the card keeps its height.
    const pieSize = computed(() => {
      const beside = (containerWidth.value || 420) - 200;
      return Math.round(Math.max(150, Math.min(beside, 190)));
    });

    const total = computed(() => props.distribution.reduce((sum, genre) => sum + genre.books, 0));

    /** The seven biggest genres, with everything below them summed into "Other". */
    const buckets = computed(() => {
      const top = props.distribution.slice(0, COLORS.length);
      const rest = props.distribution.slice(COLORS.length);
      const entries = top.map((genre, index) => ({
        key: genre.genre,
        label: genre.genre,
        value: genre.books,
        color: COLORS[index],
      }));
      if (rest.length > 0) {
        entries.push({
          key: '__other',
          label: t('stats.otherGenres', {count: rest.length}),
          value: rest.reduce((sum, genre) => sum + genre.books, 0),
          color: OTHER_COLOR,
        });
      }
      return entries;
    });

    const slices = computed(() => {
      let offset = 0;
      return buckets.value.map((bucket) => {
        const pct = (bucket.value / total.value) * 100;
        const share = Math.round(pct);
        const slice = {
          ...bucket,
          share,
          pct,
          offset,
          content: `${bucket.label}: ${bucket.value} (${share}%)`,
        };
        offset += pct;
        return slice;
      });
    });

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    return {cardEl, pieSize, total, slices, periodLabel, tooltip: useChartTooltip()};
  },
});
</script>

<style scoped>
.pie-slice {
  cursor: pointer;
}

.stat-mono {
  font-family: var(--font-mono), monospace;
  font-size: 12px;
  color: var(--color-ink-2);
}
</style>
