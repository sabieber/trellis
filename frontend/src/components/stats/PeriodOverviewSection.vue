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
          :icon="DocumentTextIcon"
          :label="$t('stats.pagesRead')"
          :value="formatNumber(stats.pages_read)"
          :subtext="$t('stats.perDay', { n: pagesPerDay })"
      />
      <StatCard
          :icon="PlusCircleIcon"
          :label="$t('stats.booksAdded')"
          :value="stats.books_added"
          :subtext="$t('stats.newInLibrary')"
      />
      <StatCard
          :icon="CalendarDaysIcon"
          :label="$t('stats.readingDays')"
          :value="stats.reading_days"
          :subtext="$t('stats.withProgress')"
      />
      <StatCard
          :icon="FireIcon"
          :label="$t('stats.dayStreak')"
          :value="stats.reading_streak_days"
          :subtext="$t('stats.daysInRow', stats.reading_streak_days)"
      />
      <StatCard
          v-if="stats.average_rating !== null"
          :icon="StarIcon"
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
  ClockIcon,
  DocumentTextIcon,
  FireIcon,
  PlusCircleIcon,
  ScaleIcon,
  StarIcon,
} from '@heroicons/vue/24/outline';
import StatCard from '@/components/stats/StatCard.vue';
import {usePeriodResource} from '@/composables/usePeriodResource';
import {formatPeriod} from '@/utils/period';
import moment from 'moment';

interface PeriodOverview {
  mode: string;
  year: number;
  month: number | null;
  books_read: number;
  pages_read: number;
  books_added: number;
  reading_days: number;
  reading_streak_days: number;
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
        props.mode === 'year' ? `${props.year}` : moment().month(props.month - 1).format('MMM'),
    );

    const pagesPerDay = computed(() => {
      if (!stats.value) return 0;
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const start = props.mode === 'year'
          ? new Date(props.year, 0, 1)
          : new Date(props.year, props.month - 1, 1);
      const end = props.mode === 'year'
          ? new Date(props.year, 11, 31)
          : new Date(props.year, props.month, 0);
      const effectiveEnd = end < today ? end : today;
      const days = Math.floor((effectiveEnd.getTime() - start.getTime()) / 86_400_000) + 1;
      return Math.round(stats.value.pages_read / days);
    });

    const formatNumber = (n: number) => n.toLocaleString();

    return {
      stats,
      loading,
      periodLabel,
      finishedInLabel,
      pagesPerDay,
      formatNumber,
      BookOpenIcon,
      CalendarDaysIcon,
      ClockIcon,
      DocumentTextIcon,
      FireIcon,
      PlusCircleIcon,
      ScaleIcon,
      StarIcon,
    };
  },
});
</script>
