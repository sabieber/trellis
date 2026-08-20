<template>
  <div>
    <div class="flex items-baseline gap-2 mb-3">
      <h2 class="t-eyebrow">{{ $t('stats.keyFacts') }}</h2>
      <span v-if="stats" class="t-meta">{{ periodLabel }}</span>
    </div>

    <div v-if="loading" class="flex justify-center py-6">
      <span class="loading loading-spinner loading-sm"></span>
    </div>

    <div v-else-if="!stats" class="t-meta text-center py-6">
      {{ $t('stats.couldNotLoad') }}
    </div>

    <div v-else class="grid grid-cols-2 md:grid-cols-3 gap-3.5">
      <StatCard
          :icon="BookOpenIcon"
          :label="$t('stats.booksRead')"
          :value="stats.books_read"
          :subtext="$t('stats.finishedIn', { label: finishedInLabel })"
      />
      <StatCard
          :icon="FileTextIcon"
          :label="$t('stats.pagesRead')"
          :value="formatNumber(stats.pages_read)"
          :subtext="$t('stats.perDay', { n: pagesPerDay })"
      />
      <StatCard
          :icon="UsersIcon"
          :label="$t('stats.authorsRead')"
          :value="stats.authors_read"
          :subtext="$t('stats.finishedIn', { label: finishedInLabel })"
      />
      <StatCard
          :icon="CalendarDaysIcon"
          :label="$t('stats.readingDays')"
          :value="stats.reading_days"
          :subtext="$t('stats.withProgress')"
      />
      <StatCard
          :icon="FlameIcon"
          :label="$t('stats.dayStreak')"
          :value="stats.reading_streak_days"
          :subtext="$t('stats.daysInRow', stats.reading_streak_days)"
      />
      <StatCard
          :icon="FlameKindlingIcon"
          :label="$t('stats.weekStreak')"
          :value="stats.reading_streak_weeks"
          :subtext="$t('stats.weeksInRow', stats.reading_streak_weeks)"
      />
      <!-- A mean of thumbs (1s and 5s) is a number without a meaning. The
           distribution card carries the share of likes instead. -->
      <StatCard
          v-if="stats.average_rating !== null && !thumbs"
          :icon="FlowerIcon"
          :label="$t('stats.avgRating')"
          :value="stats.average_rating.toFixed(1)"
          :subtext="$t('stats.ofFinished')"
      />
      <StatCard
          v-if="stats.avg_days_to_finish !== null"
          :icon="ClockIcon"
          :label="$t('stats.daysToFinish')"
          :value="stats.avg_days_to_finish.toFixed(stats.avg_days_to_finish < 10 ? 1 : 0)"
          :subtext="$t('stats.avgPerFinished')"
      />
      <StatCard
          :icon="ScaleIcon"
          :label="$t('stats.addedVsFinished')"
          :value="`${stats.books_added} / ${stats.books_read}`"
          :subtext="$t('stats.addedFinished')"
      />
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, computed, toRef} from 'vue';
import {
  BookOpenIcon,
  CalendarDaysIcon,
  FlameKindlingIcon,
  ClockIcon,
  FileTextIcon,
  FlameIcon,
  ScaleIcon,
  FlowerIcon,
  UsersIcon,
} from '@lucide/vue';
import StatCard from '@/components/stats/StatCard.vue';
import {usePeriodResource} from '@/composables/usePeriodResource';
import {ratingMode} from '@/utils/ratingMode';
import {formatPeriod} from '@/utils/period';
import moment from 'moment';

interface PeriodOverview {
  mode: string;
  year: number;
  month: number | null;
  period_start: string;
  period_end: string;
  books_read: number;
  pages_read: number;
  books_added: number;
  reading_days: number;
  authors_read: number;
  reading_streak_days: number;
  reading_streak_weeks: number;
  average_rating: number | null;
  avg_days_to_finish: number | null;
}

export default defineComponent({
  components: {StatCard},
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
  },
  setup(props) {
    const {data: stats, loading} = usePeriodResource<PeriodOverview>(
        '/api/stats/overview',
        toRef(props, 'mode'),
        toRef(props, 'year'),
        toRef(props, 'month'),
    );

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    const finishedInLabel = computed(() =>
        props.mode === 'total' ? formatPeriod('total', props.year, props.month)
            : props.mode === 'year' ? `${props.year}`
                : moment().month(props.month - 1).format('MMM'),
    );

    // Averaged over the period the server actually reported on: for total mode
    // that runs from the first reading, not some fixed calendar boundary.
    const pagesPerDay = computed(() => {
      if (!stats.value) return 0;
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const start = new Date(stats.value.period_start);
      const end = new Date(stats.value.period_end);
      const effectiveEnd = end < today ? end : today;
      const days = Math.max(1, Math.floor((effectiveEnd.getTime() - start.getTime()) / 86_400_000) + 1);
      return Math.round(stats.value.pages_read / days);
    });

    const formatNumber = (n: number) => n.toLocaleString();

    const thumbs = computed(() => ratingMode.value === 'thumbs');

    return {
      stats,
      loading,
      thumbs,
      periodLabel,
      finishedInLabel,
      pagesPerDay,
      formatNumber,
      BookOpenIcon,
      CalendarDaysIcon,
      FlameKindlingIcon,
      ClockIcon,
      FileTextIcon,
      FlameIcon,
      ScaleIcon,
      FlowerIcon,
      UsersIcon,
    };
  },
});
</script>
