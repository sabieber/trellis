<!-- Bar chart of the reading volume over the period: months in year mode, days
     in month mode. The metric toggle switches between pages and finished books
     without refetching, both come from the same activity series. -->
<template>
  <div>
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ metric === 'pages' ? $t('stats.pagesRead') : $t('stats.booksRead') }}</h2>
        <span class="t-meta">{{ bucketLabel }}</span>
      </div>
      <span v-if="!loading && total > 0" class="t-meta">{{ summary }}</span>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 flex flex-col min-w-0">
      <div ref="chartEl" class="w-full flex-1 relative" :style="{minHeight: `${HEIGHT}px`}">
        <div v-if="loading" class="flex justify-center py-14">
          <span class="loading loading-spinner loading-sm"></span>
        </div>

        <div v-else-if="!series" class="t-meta text-center py-14">
          {{ $t('stats.couldNotLoadActivity') }}
        </div>

        <div v-else-if="total === 0" class="t-meta text-center py-14">
          {{ metric === 'pages' ? $t('stats.noPagesLogged') : $t('stats.noBooksLogged') }}
        </div>

        <svg v-else :viewBox="`0 0 ${width} ${height}`" preserveAspectRatio="none" class="absolute inset-0 w-full h-full block">
          <g>
            <line
                v-for="tick in ticks"
                :key="`grid-${tick.value}`"
                :x1="padLeft"
                :x2="width - padRight"
                :y1="tick.y"
                :y2="tick.y"
                :class="tick.value === 0 ? 'axis-line' : 'grid-line'"
            />
            <text
                v-for="tick in ticks"
                :key="`label-${tick.value}`"
                :x="padLeft - 8"
                :y="tick.y"
                text-anchor="end"
                dominant-baseline="middle"
                class="axis-text"
            >{{ tick.label }}
            </text>
          </g>

          <g>
            <path
                v-for="bar in filledBars"
                :key="`bar-${bar.key}`"
                :d="bar.path"
                class="bar"
                v-on="tooltip.marks(() => bar.title)"
            />
          </g>

          <g>
            <text
                v-for="bar in labelledBars"
                :key="`tick-${bar.key}`"
                :x="bar.bandX + bar.bandWidth / 2"
                :y="height - 6"
                text-anchor="middle"
                class="axis-text"
            >{{ bar.label }}
            </text>
          </g>
        </svg>
      </div>
    </div>

    <div class="flex justify-end mt-3">
      <SegmentedControl
          v-model="metric"
          :options="[{value: 'pages', label: $t('common.pages')}, {value: 'books', label: $t('common.books')}]"
          class="w-36"
      />
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, onMounted, ref, type PropType} from 'vue';
import {useI18n} from 'vue-i18n';
import moment from 'moment';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {useChartTooltip} from '@/composables/useChartTooltip';
import {daysInMonth} from '@/utils/activityHeat';
import {niceScale} from '@/utils/niceScale';
import type {ActivitySeries} from '@/composables/useActivityStats';

const HEIGHT = 208;
const PAD_LEFT = 36;
const PAD_RIGHT = 4;
const PAD_TOP = 10;
const PAD_BOTTOM = 24;

/** Path of a bar with rounded top corners. */
function barPath(x: number, y: number, barWidth: number, barHeight: number): string {
    const radius = Math.min(2.5, barWidth / 2, barHeight);
    const bottom = y + barHeight;
    return [
        `M${x} ${bottom}`,
        `V${y + radius}`,
        `Q${x} ${y} ${x + radius} ${y}`,
        `H${x + barWidth - radius}`,
        `Q${x + barWidth} ${y} ${x + barWidth} ${y + radius}`,
        `V${bottom}`,
        'Z',
    ].join(' ');
}

export default defineComponent({
    components: {SegmentedControl},
    props: {
        mode: {
            type: String,
            required: true,
        },
        year: {
            type: Number,
            required: true,
        },
        month: {
            type: Number,
            required: true,
        },
        series: {
            type: Object as PropType<ActivitySeries | null>,
            default: null,
        },
        loading: {
            type: Boolean,
            default: false,
        },
    },
    setup(props) {
        const {t} = useI18n();
        const metric = ref<'pages' | 'books'>('pages');
        const tooltip = useChartTooltip();
        const chartEl = ref<HTMLElement | null>(null);
        const {containerWidth, containerHeight, setupObserver} = useContainerWidth(chartEl);

        onMounted(setupObserver);

        const width = computed(() => Math.max(280, containerWidth.value || 640));
        const plotWidth = computed(() => width.value - PAD_LEFT - PAD_RIGHT);
        // The card is stretched to match its sibling via subgrid; the chart fills it.
        const height = computed(() => Math.max(HEIGHT, Math.round(containerHeight.value) || HEIGHT));
        const plotHeight = computed(() => height.value - PAD_TOP - PAD_BOTTOM);

        /** First calendar year of the span, in total mode. Anchors the yearly
         *  buckets; falls back to this year when there is no activity yet. */
        const firstYear = computed(() =>
            props.series ? Number(props.series.start.slice(0, 4)) : props.year,
        );

        /** One bucket per month of the year, per day of the month, or — in total
         *  mode — per calendar year from the first activity to today. */
        const buckets = computed(() => {
            const count = props.mode === 'total'
                ? Number((props.series?.end ?? '').slice(0, 4)) - firstYear.value + 1 || 1
                : props.mode === 'year' ? 12 : daysInMonth(props.year, props.month);
            const values = new Array<number>(Math.max(count, 1)).fill(0);

            for (const day of props.series?.days ?? []) {
                const [year, month, dayOfMonth] = day.date.split('-').map(Number);
                const index = props.mode === 'total'
                    ? year - firstYear.value
                    : props.mode === 'year' ? month - 1 : dayOfMonth - 1;
                if (index >= 0 && index < values.length) {
                    values[index] += metric.value === 'pages' ? day.pages : day.books;
                }
            }
            return values;
        });

        const total = computed(() => buckets.value.reduce((sum, value) => sum + value, 0));

        const scale = computed(() =>
            niceScale(Math.max(...buckets.value, 0), metric.value === 'books'),
        );

        const ticks = computed(() => {
            const {max, step} = scale.value;
            const result: { value: number; label: string; y: number }[] = [];
            for (let value = 0; value <= max + step / 2; value += step) {
                result.push({
                    value,
                    label: value.toLocaleString(),
                    y: PAD_TOP + plotHeight.value - (value / max) * plotHeight.value,
                });
            }
            return result;
        });

        const bars = computed(() => {
            const isYear = props.mode === 'year';
            const isTotal = props.mode === 'total';
            const values = buckets.value;
            const bandWidth = plotWidth.value / values.length;
            const barWidth = Math.max(2, Math.min(bandWidth - 3, 20));
            // Keep the day/year axis readable: label roughly every eighth bucket.
            const labelEvery = Math.max(1, Math.ceil(values.length / 8));
            const unitKey = metric.value === 'pages' ? 'stats.unitPages' : 'stats.unitBooks';

            return values.map((value, index) => {
                const bandX = PAD_LEFT + bandWidth * index;
                const barHeight = (value / scale.value.max) * plotHeight.value;
                const monthLabel = moment().month(index).format('MMM');
                const label = isTotal
                    ? index % labelEvery === 0 ? `${firstYear.value + index}` : ''
                    : isYear
                        ? bandWidth >= 26 ? monthLabel : monthLabel.charAt(0)
                        : index % labelEvery === 0 ? `${index + 1}` : '';
                const bucketName = isTotal
                    ? `${firstYear.value + index}`
                    : isYear
                        ? moment().year(props.year).month(index).format('MMMM YYYY')
                        : moment().year(props.year).month(props.month - 1).date(index + 1).format('D MMM YYYY');

                return {
                    key: index,
                    bandX,
                    bandWidth,
                    label,
                    path: value > 0
                        ? barPath(bandX + (bandWidth - barWidth) / 2, PAD_TOP + plotHeight.value - barHeight, barWidth, barHeight)
                        : '',
                    title: `${bucketName} — ${value.toLocaleString()} ${t(unitKey, value)}`,
                };
            });
        });

        const filledBars = computed(() => bars.value.filter((bar) => bar.path !== ''));
        const labelledBars = computed(() => bars.value.filter((bar) => bar.label !== ''));

        const bucketLabel = computed(() =>
            props.mode === 'total'
                ? t('stats.byYear')
                : props.mode === 'year'
                    ? t('stats.byMonthIn', {year: props.year})
                    : t('stats.byDayIn', {label: moment().year(props.year).month(props.month - 1).format('MMM YYYY')}),
        );

        const perUnitKey = computed(() =>
            props.mode === 'total' ? 'stats.perYearUnit'
                : props.mode === 'year' ? 'stats.perMonthUnit' : 'stats.perDayUnit',
        );

        const summary = computed(() => {
            const active = buckets.value.filter((value) => value > 0).length || 1;
            return t('stats.volumeSummary', {
                total: total.value.toLocaleString(),
                unit: t(metric.value === 'pages' ? 'stats.unitPages' : 'stats.unitBooks', total.value),
                perActive: Math.round(total.value / active).toLocaleString(),
                per: t(perUnitKey.value),
            });
        });

        return {
            metric,
            tooltip,
            chartEl,
            HEIGHT,
            width,
            height,
            padLeft: PAD_LEFT,
            padRight: PAD_RIGHT,
            padTop: PAD_TOP,
            plotHeight,
            ticks,
            bars,
            filledBars,
            labelledBars,
            total,
            bucketLabel,
            summary,
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

.bar {
    fill: #93c456;
    cursor: pointer;
    transition: fill 120ms ease;
}

.bar:hover {
    fill: #a7d06e;
}
</style>
