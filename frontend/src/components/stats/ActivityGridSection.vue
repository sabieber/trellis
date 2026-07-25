<!-- GitHub-style reading heatmap. In year mode it lays the year out as week
     columns, in month mode as a calendar of the selected month. Intensity
     always encodes the pages logged on that day. -->
<template>
  <div>
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t('stats.activityTitle') }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
      <span v-if="!loading" class="t-meta">{{ summary }}</span>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 min-w-0">
      <div ref="gridEl" :style="{'--cell': `${cellSize}px`}">
        <div v-if="loading" class="flex justify-center py-8">
          <span class="loading loading-spinner loading-sm"></span>
        </div>

        <div v-else-if="!series" class="t-meta text-center py-8">
          {{ $t('stats.couldNotLoadActivity') }}
        </div>

        <!-- Year: 7 weekday rows × week columns -->
        <div v-else-if="mode === 'year'" ref="scrollEl" class="overflow-x-auto pb-1">
          <div class="w-max">
            <div class="grid ml-[34px] mb-1.5" :style="columnStyle">
              <span
                  v-for="(span, index) in monthSpans"
                  :key="index"
                  class="heat-axis"
                  :style="{gridColumn: `span ${span.span}`}"
              >{{ span.span >= 3 ? span.label : '' }}</span>
            </div>

            <div class="flex gap-1.5">
              <div class="grid gap-[3px] w-7 shrink-0">
                <span v-for="(label, index) in weekdayRowLabels" :key="index" class="heat-axis heat-axis-row">
                  {{ label }}
                </span>
              </div>
              <div class="grid grid-flow-col grid-rows-7 gap-[3px]">
                <div
                    v-for="cell in yearCells"
                    :key="cell.id"
                    class="heat-cell"
                    :class="cellClass(cell)"
                    v-on="tooltip.marks(() => cell.title)"
                ></div>
              </div>
            </div>
          </div>
        </div>

        <!-- Month: calendar with the day numbers spelled out -->
        <div v-else class="max-w-sm">
          <div class="grid grid-cols-7 gap-1.5 mb-1.5">
            <span v-for="(label, index) in weekdayShortLabels" :key="index" class="heat-axis text-center">
              {{ label }}
            </span>
          </div>
          <div class="grid grid-cols-7 gap-1.5">
            <div
                v-for="cell in monthCells"
                :key="cell.id"
                class="heat-day"
                :class="cellClass(cell)"
                v-on="tooltip.marks(() => cell.title)"
            >
              <span v-if="!cell.placeholder">{{ cell.dayOfMonth }}</span>
            </div>
          </div>
        </div>

        <div v-if="series && !loading" class="flex items-center justify-end gap-1.5 mt-3.5" style="--cell: 11px">
          <span class="t-meta">{{ $t('stats.less') }}</span>
          <div v-for="level in [0, 1, 2, 3, 4]" :key="level" class="heat-cell" :class="`heat-${level}`"></div>
          <span class="t-meta">{{ $t('stats.more') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, nextTick, onMounted, ref, watch, type PropType} from 'vue';
import {useI18n} from 'vue-i18n';
import moment from 'moment';
import type {ActivityDay, ActivitySeries} from '@/composables/useActivityStats';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {useChartTooltip} from '@/composables/useChartTooltip';
import {dateKey, daysInMonth, heatLevel, heatThresholds, mondayIndex} from '@/utils/activityHeat';
import {formatPeriod} from '@/utils/period';

/** Width of the weekday label column plus its gap towards the grid. */
const LABEL_GUTTER = 34;
const CELL_GAP = 3;
const CELL_MIN = 11;
const CELL_MAX = 18;

interface HeatCell {
  id: string;
  dayOfMonth: number;
  level: number;
  future: boolean;
  placeholder: boolean;
  title: string;
}

export default defineComponent({
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
    byDate: {
      type: Object as PropType<Map<string, ActivityDay>>,
      required: true,
    },
    loading: {
      type: Boolean,
      default: false,
    },
  },
  setup(props) {
    const {t, locale} = useI18n();
    const tooltip = useChartTooltip();
    const today = dateKey(new Date());

    // Localized heatmap axis labels (Mon–Sun); recompute when the UI locale changes.
    const weekdayRowLabels = computed(() =>
        Array.from({length: 7}, (_, i) => (locale.value, i % 2 === 0 ? moment().isoWeekday(i + 1).format('ddd') : '')));
    const weekdayShortLabels = computed(() =>
        Array.from({length: 7}, (_, i) => (locale.value, moment().isoWeekday(i + 1).format('dd').charAt(0))));
    const gridEl = ref<HTMLElement | null>(null);
    const scrollEl = ref<HTMLElement | null>(null);
    const {containerWidth, setupObserver} = useContainerWidth(gridEl);

    onMounted(setupObserver);

    const thresholds = computed(() =>
        heatThresholds((props.series?.days ?? []).map((day) => day.pages)),
    );

    const placeholder = (id: string): HeatCell => ({
      id,
      dayOfMonth: 0,
      level: 0,
      future: false,
      placeholder: true,
      title: '',
    });

    const cellFor = (date: Date): HeatCell => {
      const key = dateKey(date);
      const activity = props.byDate.get(key);
      const pages = activity?.pages ?? 0;
      const books = activity?.books ?? 0;

      const parts: string[] = [];
      if (pages > 0) parts.push(`${pages.toLocaleString()} ${t('stats.unitPages', pages)}`);
      if (books > 0) parts.push(`${books} ${t('stats.booksFinished', books)}`);
      const detail = parts.length ? parts.join(', ') : t('stats.noReadingLogged');

      return {
        id: key,
        dayOfMonth: date.getDate(),
        level: heatLevel(pages, thresholds.value),
        future: key > today,
        placeholder: false,
        title: `${moment(date).format('ddd, D MMM YYYY')} — ${detail}`,
      };
    };

    /** Week columns of the whole year, flowing top-to-bottom then left-to-right. */
    const yearCells = computed(() => {
      const cells: HeatCell[] = [];
      const lastDay = new Date(props.year, 11, 31);
      const cursor = new Date(props.year, 0, 1);
      cursor.setDate(cursor.getDate() - mondayIndex(cursor));

      while (cursor <= lastDay) {
        for (let offset = 0; offset < 7; offset++) {
          const date = new Date(cursor);
          date.setDate(date.getDate() + offset);
          cells.push(
              date.getFullYear() === props.year
                  ? cellFor(date)
                  : placeholder(`pad-${dateKey(date)}`),
          );
        }
        cursor.setDate(cursor.getDate() + 7);
      }
      return cells;
    });

    const weekColumnCount = computed(() => yearCells.value.length / 7);

    /** Cells grow to fill the card and only fall back to scrolling below the minimum. */
    const cellSize = computed(() => {
      const columns = weekColumnCount.value;
      const available = (containerWidth.value || 760) - LABEL_GUTTER - CELL_GAP * (columns - 1);
      return Math.min(CELL_MAX, Math.max(CELL_MIN, Math.floor(available / columns)));
    });

    const columnStyle = computed(() => ({
      gridTemplateColumns: `repeat(${weekColumnCount.value}, ${cellSize.value}px)`,
      columnGap: `${CELL_GAP}px`,
    }));

    // The running year is worth more than January: when the grid has to scroll,
    // park today's column at the right edge instead of the empty year end.
    const scrollToToday = () => {
      const element = scrollEl.value;
      const now = new Date();
      if (!element || props.mode !== 'year' || now.getFullYear() !== props.year) return;

      const gridStart = new Date(props.year, 0, 1);
      gridStart.setDate(gridStart.getDate() - mondayIndex(gridStart));
      const midnight = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      const daysIn = Math.round((midnight.getTime() - gridStart.getTime()) / 86_400_000);
      const rightEdge = LABEL_GUTTER + (Math.floor(daysIn / 7) + 1) * (cellSize.value + CELL_GAP);
      element.scrollLeft = Math.max(0, rightEdge - element.clientWidth + 4);
    };

    watch(
        () => [props.mode, props.year, props.series, cellSize.value],
        async () => {
          await nextTick();
          scrollToToday();
        },
        {immediate: true},
    );

    /** Month labels spanning the week columns they own the most days of. */
    const monthSpans = computed(() => {
      const spans: { label: string; span: number }[] = [];
      const cursor = new Date(props.year, 0, 1);
      cursor.setDate(cursor.getDate() - mondayIndex(cursor));

      for (let column = 0; column < weekColumnCount.value; column++) {
        // The Thursday decides: it belongs to the month owning most of the week.
        const midweek = new Date(cursor);
        midweek.setDate(midweek.getDate() + column * 7 + 3);
        const label = moment(midweek).format('MMM');
        const previous = spans[spans.length - 1];
        if (previous && previous.label === label) {
          previous.span++;
        } else {
          spans.push({label, span: 1});
        }
      }
      return spans;
    });

    /** Calendar cells of the selected month, padded to full weeks up front. */
    const monthCells = computed(() => {
      const first = new Date(props.year, props.month - 1, 1);
      const cells: HeatCell[] = [];
      for (let index = 0; index < mondayIndex(first); index++) {
        cells.push(placeholder(`pad-${index}`));
      }
      for (let day = 1; day <= daysInMonth(props.year, props.month); day++) {
        cells.push(cellFor(new Date(props.year, props.month - 1, day)));
      }
      return cells;
    });

    const cellClass = (cell: HeatCell) => {
      if (cell.placeholder) return 'heat-placeholder';
      if (cell.future) return 'heat-future';
      return `heat-${cell.level}`;
    };

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    const summary = computed(() => {
      const days = props.series?.days ?? [];
      const pages = days.reduce((sum, day) => sum + day.pages, 0);
      const books = days.reduce((sum, day) => sum + day.books, 0);
      if (pages === 0 && books === 0) return t('stats.nothingLogged');
      return t('stats.activitySummary', {
        pages: pages.toLocaleString(),
        books: t('stats.booksFinished', books),
      });
    });

    return {
      gridEl,
      scrollEl,
      cellSize,
      yearCells,
      monthCells,
      monthSpans,
      columnStyle,
      cellClass,
      periodLabel,
      summary,
      tooltip,
      weekdayRowLabels,
      weekdayShortLabels,
    };
  },
});
</script>

<style scoped>
/* Heat ramp interpolated from the empty surface towards the accent green.
   Kept as literal hex like the other visual primitives, since these tints
   are chart-local and not part of the shared token set. */
.heat-cell {
  width: var(--cell, 12px);
  height: var(--cell, 12px);
  border-radius: 3px;
}

.heat-day {
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  aspect-ratio: 1;
  padding: 5px;
  border-radius: 5px;
  font-family: var(--font-mono), monospace;
  font-size: 11px;
  line-height: 1;
  color: #b6ab8f;
}

.heat-0 {
  background: #2a2619;
}

.heat-1 {
  background: #3b4a25;
}

.heat-2 {
  background: #556d2f;
}

.heat-3 {
  background: #74993c;
}

.heat-4 {
  background: #93c456;
}

.heat-3,
.heat-4 {
  color: #16170d;
}

.heat-future {
  background: #201d15;
}

.heat-placeholder {
  background: transparent;
}

.heat-axis {
  font-family: var(--font-mono), monospace;
  font-size: 10px;
  line-height: var(--cell, 12px);
  color: #8f866f;
  white-space: nowrap;
}

.heat-axis-row {
  height: var(--cell, 12px);
}
</style>
