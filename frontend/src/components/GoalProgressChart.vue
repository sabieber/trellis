<!-- Cumulative progress toward a reading goal across its period: one point per
     month (year goals) or per day (month goals). The rising area climbs toward a
     dashed target line, so the chart reads as the companion to the % progress
     bar above it. Data comes from the same /api/stats/activity series the stats
     charts trust, so the curve lands on the same figure the progress bar shows. -->
<template>
  <div>
    <div class="flex items-baseline gap-2 mb-3">
      <h2 class="t-eyebrow">{{ $t('goalDetail.progressChart') }}</h2>
      <span class="t-meta">{{ bucketLabel }}</span>
    </div>

    <div class="bg-surface border border-line rounded-md p-4">
      <div ref="chartEl" class="w-full relative" :style="{minHeight: `${HEIGHT}px`}">
        <div v-if="loading" class="flex justify-center py-14">
          <span class="loading loading-spinner loading-sm"></span>
        </div>

        <svg v-else :viewBox="`0 0 ${width} ${height}`" preserveAspectRatio="none" class="absolute inset-0 w-full h-full block">
          <g>
            <line
                v-for="tick in ticks"
                :key="`grid-${tick.value}`"
                :x1="padLeft" :x2="width - padRight" :y1="tick.y" :y2="tick.y"
                :class="tick.value === 0 ? 'axis-line' : 'grid-line'"
            />
            <text
                v-for="tick in ticks"
                :key="`label-${tick.value}`"
                :x="padLeft - 8" :y="tick.y"
                text-anchor="end" dominant-baseline="middle" class="axis-text"
            >{{ tick.label }}</text>
          </g>

          <!-- Target reference line -->
          <line
              :x1="padLeft" :x2="width - padRight" :y1="targetY" :y2="targetY"
              class="target-line"
          />
          <text :x="width - padRight" :y="targetY - 4" text-anchor="end" class="target-text">
            {{ $t('goalDetail.goalTarget', {target: target.toLocaleString()}) }}
          </text>

          <path :d="areaPath" class="area"/>
          <path :d="linePath" class="line"/>
          <circle :cx="currentPoint.x" :cy="currentPoint.y" r="3" class="dot"/>

          <!-- Transparent per-bucket hit bands drive the shared tooltip. -->
          <rect
              v-for="band in hitBands"
              :key="`band-${band.key}`"
              :x="band.x" :y="padTop" :width="band.width" :height="plotHeight"
              fill="transparent"
              v-on="tooltip.marks(() => band.title)"
          />

          <g>
            <text
                v-for="label in axisLabels"
                :key="`x-${label.key}`"
                :x="label.x" :y="height - 6"
                text-anchor="middle" class="axis-text"
            >{{ label.text }}</text>
          </g>
        </svg>
      </div>
    </div>

    <ChartTooltip/>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, onMounted, ref, type PropType} from 'vue';
import {useI18n} from 'vue-i18n';
import moment from 'moment';
import ChartTooltip from '@/components/stats/ChartTooltip.vue';
import {usePeriodResource} from '@/composables/usePeriodResource';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {useChartTooltip} from '@/composables/useChartTooltip';
import {daysInMonth} from '@/utils/activityHeat';
import {niceScale} from '@/utils/niceScale';
import type {ActivitySeries} from '@/composables/useActivityStats';

interface Goal {
  goal_type: string;
  timeframe: string;
  target: number;
  period_start: string;
}

const HEIGHT = 208;
const PAD_LEFT = 36;
const PAD_RIGHT = 4;
const PAD_TOP = 10;
const PAD_BOTTOM = 24;

export default defineComponent({
  components: {ChartTooltip},
  props: {
    goal: {
      type: Object as PropType<Goal>,
      required: true,
    },
  },
  setup(props) {
    const {t} = useI18n();
    const tooltip = useChartTooltip();
    const chartEl = ref<HTMLElement | null>(null);
    const {containerWidth, containerHeight, setupObserver} = useContainerWidth(chartEl);
    onMounted(setupObserver);

    const year = computed(() => Number(props.goal.period_start.slice(0, 4)));
    const month = computed(() => Number(props.goal.period_start.slice(5, 7)));
    // The chart is only mounted for year/month goals (see GoalDetailView).
    const mode = computed(() => (props.goal.timeframe === 'year' ? 'year' : 'month'));
    const isBooks = computed(() => props.goal.goal_type === 'books');
    const target = computed(() => props.goal.target);

    const {data: series, loading} = usePeriodResource<ActivitySeries>(
        '/api/stats/activity', mode, year, month,
    );

    const count = computed(() =>
        mode.value === 'year' ? 12 : daysInMonth(year.value, month.value),
    );

    /** Cumulative metric value at the end of each bucket. */
    const cumulative = computed(() => {
      const perBucket = new Array<number>(count.value).fill(0);
      for (const day of series.value?.days ?? []) {
        const [, m, d] = day.date.split('-').map(Number);
        const index = mode.value === 'year' ? m - 1 : d - 1;
        if (index >= 0 && index < perBucket.length) {
          perBucket[index] += isBooks.value ? day.books : day.pages;
        }
      }
      let running = 0;
      return perBucket.map((value) => (running += value));
    });

    // Goals always cover the current period, so today's bucket is the last one
    // with real data; the curve stops there rather than running flat to the end.
    const nowIndex = computed(() => {
      const now = new Date();
      const raw = mode.value === 'year' ? now.getMonth() : now.getDate() - 1;
      return Math.min(Math.max(raw, 0), count.value - 1);
    });

    const current = computed(() => cumulative.value[nowIndex.value] ?? 0);

    const width = computed(() => Math.max(280, containerWidth.value || 640));
    const height = computed(() => Math.max(HEIGHT, Math.round(containerHeight.value) || HEIGHT));
    const plotWidth = computed(() => width.value - PAD_LEFT - PAD_RIGHT);
    const plotHeight = computed(() => height.value - PAD_TOP - PAD_BOTTOM);

    const scale = computed(() => niceScale(Math.max(target.value, current.value), isBooks.value));

    const yOf = (value: number) => PAD_TOP + plotHeight.value - (value / scale.value.max) * plotHeight.value;
    // Right edge of bucket i: cumulative value is "as of the end of" that bucket.
    const xOf = (index: number) => PAD_LEFT + ((index + 1) / count.value) * plotWidth.value;
    const baseY = computed(() => yOf(0));

    const ticks = computed(() => {
      const {max, step} = scale.value;
      const result: { value: number; label: string; y: number }[] = [];
      for (let value = 0; value <= max + step / 2; value += step) {
        result.push({value, label: value.toLocaleString(), y: yOf(value)});
      }
      return result;
    });

    const targetY = computed(() => yOf(target.value));

    const points = computed(() => {
      const pts = [{x: PAD_LEFT, y: baseY.value}];
      for (let i = 0; i <= nowIndex.value; i++) {
        pts.push({x: xOf(i), y: yOf(cumulative.value[i])});
      }
      return pts;
    });

    const currentPoint = computed(() => points.value[points.value.length - 1]);

    const linePath = computed(() =>
        points.value.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x} ${p.y}`).join(' '),
    );

    const areaPath = computed(() => {
      const last = currentPoint.value;
      return `${linePath.value} L${last.x} ${baseY.value} Z`;
    });

    const unitKey = computed(() => (isBooks.value ? 'stats.unitBooks' : 'stats.unitPages'));

    const bucketName = (index: number) =>
        mode.value === 'year'
            ? moment().year(year.value).month(index).format('MMMM YYYY')
            : moment().year(year.value).month(month.value - 1).date(index + 1).format('D MMM YYYY');

    const hitBands = computed(() => {
      const bandWidth = plotWidth.value / count.value;
      const bands: { key: number; x: number; width: number; title: string }[] = [];
      for (let i = 0; i <= nowIndex.value; i++) {
        const value = cumulative.value[i];
        bands.push({
          key: i,
          x: PAD_LEFT + bandWidth * i,
          width: bandWidth,
          title: t('goalDetail.progressAt', {
            bucket: bucketName(i),
            value: value.toLocaleString(),
            target: target.value.toLocaleString(),
            unit: t(unitKey.value, target.value),
          }),
        });
      }
      return bands;
    });

    const axisLabels = computed(() => {
      const bandWidth = plotWidth.value / count.value;
      const labelEvery = Math.max(1, Math.ceil(count.value / 8));
      const labels: { key: number; x: number; text: string }[] = [];
      for (let i = 0; i < count.value; i++) {
        const monthLabel = moment().month(i).format('MMM');
        const text = mode.value === 'year'
            ? (bandWidth >= 26 ? monthLabel : monthLabel.charAt(0))
            : (i % labelEvery === 0 ? `${i + 1}` : '');
        if (text) labels.push({key: i, x: PAD_LEFT + bandWidth * (i + 0.5), text});
      }
      return labels;
    });

    const bucketLabel = computed(() =>
        mode.value === 'year'
            ? t('stats.byMonthIn', {year: year.value})
            : t('stats.byDayIn', {label: moment().year(year.value).month(month.value - 1).format('MMM YYYY')}),
    );

    return {
      HEIGHT, loading, tooltip, chartEl,
      width, height, padLeft: PAD_LEFT, padRight: PAD_RIGHT, padTop: PAD_TOP, plotHeight,
      target, ticks, targetY, areaPath, linePath, currentPoint, hitBands, axisLabels, bucketLabel,
    };
  },
});
</script>

<style scoped>
.grid-line {
  stroke: rgb(236 226 204 / 0.07);
  stroke-width: 1;
}

.axis-line {
  stroke: #38321f;
  stroke-width: 1;
}

.axis-text {
  font-family: var(--font-mono), monospace;
  font-size: 10px;
  fill: #8f866f;
}

.target-line {
  stroke: #8f866f;
  stroke-width: 1;
  stroke-dasharray: 4 3;
  opacity: 0.6;
}

.target-text {
  font-family: var(--font-mono), monospace;
  font-size: 10px;
  fill: #8f866f;
}

.area {
  fill: #93c456;
  opacity: 0.16;
}

.line {
  fill: none;
  stroke: #93c456;
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.dot {
  fill: #a7d06e;
}
</style>
