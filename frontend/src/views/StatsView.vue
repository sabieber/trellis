<template>
  <PageContainer title="Statistics" description="Your reading at a glance" wide>
    <div class="flex flex-wrap items-center justify-between gap-3 mb-7">
      <SegmentedControl
          :model-value="mode"
          @update:model-value="setMode"
          :options="[{value: 'year', label: 'Year'}, {value: 'month', label: 'Month'}]"
          class="w-44"
      />
      <div class="flex items-center gap-1.5">
        <button
            class="flex items-center justify-center size-8 rounded-full text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
            aria-label="Previous period"
            @click="step(-1)"
        >
          <ChevronLeftIcon class="size-4.5"/>
        </button>
        <span class="t-title text-base min-w-24 text-center select-none">{{ stepperLabel }}</span>
        <button
            class="flex items-center justify-center size-8 rounded-full text-muted transition-colors duration-150"
            :class="atCurrentPeriod ? 'opacity-40 cursor-default' : 'cursor-pointer hover:text-ink hover:bg-surface-2'"
            aria-label="Next period"
            :disabled="atCurrentPeriod"
            @click="step(1)"
        >
          <ChevronRightIcon class="size-4.5"/>
        </button>
      </div>
    </div>

    <div class="flex flex-col gap-7">
      <PeriodOverviewSection v-bind="period"/>
      <!-- Year gives the heatmap the wider 2/3 slot, month gives it to the bar chart.
           Both cards span the same three subgrid rows (header · card · footer) so their
           tops and bottoms line up even though only the bar chart carries a footer. -->
      <div class="grid grid-cols-1 gap-7 lg:grid-cols-3 lg:grid-rows-[auto_1fr_auto] lg:gap-y-0">
        <ActivityGridSection
            class="min-w-0 lg:row-span-3 lg:grid lg:grid-rows-subgrid"
            :class="mode === 'year' ? 'lg:col-span-2' : 'lg:col-span-1'"
            v-bind="period"
            :series="series"
            :by-date="byDate"
            :loading="activityLoading"
        />
        <ReadingVolumeSection
            class="min-w-0 lg:row-span-3 lg:grid lg:grid-rows-subgrid"
            :class="mode === 'year' ? 'lg:col-span-1' : 'lg:col-span-2'"
            v-bind="period"
            :series="series"
            :loading="activityLoading"
        />
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-7">
        <RatingDistributionSection
            v-bind="period"
            :distribution="breakdown?.rating_distribution ?? []"
            :loading="breakdownLoading"
        />
        <WeekdaySection
            v-bind="period"
            :weekday-pages="weekdayPages"
            :loading="activityLoading"
        />
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-7">
        <ReadingStatesSection
            v-bind="period"
            :states="breakdown?.reading_states ?? null"
            :loading="breakdownLoading"
        />
        <TopAuthorsSection
            v-bind="period"
            :authors="breakdown?.top_authors ?? []"
            :loading="breakdownLoading"
        />
      </div>
    </div>
    <ChartTooltip/>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed} from 'vue';
import {ChevronLeftIcon, ChevronRightIcon} from '@heroicons/vue/24/outline';
import PageContainer from '@/components/PageContainer.vue';
import ActivityGridSection from '@/components/stats/ActivityGridSection.vue';
import PeriodOverviewSection from '@/components/stats/PeriodOverviewSection.vue';
import ReadingVolumeSection from '@/components/stats/ReadingVolumeSection.vue';
import RatingDistributionSection from '@/components/stats/RatingDistributionSection.vue';
import WeekdaySection from '@/components/stats/WeekdaySection.vue';
import ReadingStatesSection from '@/components/stats/ReadingStatesSection.vue';
import TopAuthorsSection from '@/components/stats/TopAuthorsSection.vue';
import ChartTooltip from '@/components/stats/ChartTooltip.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import {useActivityStats} from '@/composables/useActivityStats';
import {useStatsBreakdown} from '@/composables/useStatsBreakdown';
import {formatPeriod} from '@/utils/period';

export default defineComponent({
  components: {
    PageContainer,
    ActivityGridSection,
    PeriodOverviewSection,
    ReadingVolumeSection,
    RatingDistributionSection,
    WeekdaySection,
    ReadingStatesSection,
    TopAuthorsSection,
    ChartTooltip,
    SegmentedControl,
    ChevronLeftIcon,
    ChevronRightIcon,
  },
  setup() {
    const now = new Date();
    const currentYear = now.getFullYear();
    const currentMonth = now.getMonth() + 1;

    const mode = ref<'year' | 'month'>('year');
    const year = ref(currentYear);
    const month = ref(currentMonth);

    const setMode = (value: string) => {
      mode.value = value as 'year' | 'month';
      if (mode.value === 'month') {
        month.value = year.value === currentYear ? currentMonth : 1;
      }
    };

    const atCurrentPeriod = computed(() =>
        mode.value === 'year'
            ? year.value >= currentYear
            : year.value > currentYear || (year.value === currentYear && month.value >= currentMonth),
    );

    const step = (direction: number) => {
      if (direction > 0 && atCurrentPeriod.value) return;
      if (mode.value === 'year') {
        year.value += direction;
      } else {
        const date = new Date(year.value, month.value - 1 + direction, 1);
        year.value = date.getFullYear();
        month.value = date.getMonth() + 1;
      }
    };

    const stepperLabel = computed(() => formatPeriod(mode.value, year.value, month.value));

    /** The period trio, spread onto every section via `v-bind`. */
    const period = computed(() => ({mode: mode.value, year: year.value, month: month.value}));

    const {series, byDate, loading: activityLoading} = useActivityStats(mode, year, month);
    const {breakdown, loading: breakdownLoading} = useStatsBreakdown(mode, year, month);

    /** Pages bucketed by weekday (Mon=0..Sun=6), derived from the activity series
     *  already in memory rather than refetched from the server. */
    const weekdayPages = computed(() => {
      const buckets = new Array<number>(7).fill(0);
      for (const day of series.value?.days ?? []) {
        const [y, m, d] = day.date.split('-').map(Number);
        const weekday = (new Date(y, m - 1, d).getDay() + 6) % 7;
        buckets[weekday] += day.pages;
      }
      return buckets;
    });

    return {
      mode,
      year,
      month,
      period,
      setMode,
      step,
      stepperLabel,
      atCurrentPeriod,
      series,
      byDate,
      activityLoading,
      breakdown,
      breakdownLoading,
      weekdayPages,
    };
  },
});
</script>
