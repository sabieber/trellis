<template>
  <div>
    <div class="flex items-baseline gap-2 mb-3">
      <h2 class="t-eyebrow">Key facts</h2>
      <span v-if="stats" class="t-meta">{{ periodLabel }}</span>
    </div>

    <div v-if="loading" class="flex justify-center py-6">
      <span class="loading loading-spinner loading-sm"></span>
    </div>

    <div v-else-if="!stats" class="t-meta text-center py-6">
      Could not load statistics.
    </div>

    <div v-else class="grid grid-cols-2 md:grid-cols-3 gap-3.5">
      <StatCard
          :icon="BookOpenIcon"
          label="Books read"
          :value="stats.books_read"
          :subtext="`finished in ${finishedInLabel}`"
      />
      <StatCard
          :icon="DocumentTextIcon"
          label="Pages read"
          :value="formatNumber(stats.pages_read)"
          :subtext="`~${pagesPerDay} per day`"
      />
      <StatCard
          :icon="PlusCircleIcon"
          label="Books added"
          :value="stats.books_added"
          subtext="new in your library"
      />
      <StatCard
          :icon="CalendarDaysIcon"
          label="Reading days"
          :value="stats.reading_days"
          subtext="with logged progress"
      />
      <StatCard
          :icon="FireIcon"
          label="Day streak"
          :value="stats.reading_streak_days"
          :subtext="stats.reading_streak_days === 1 ? 'day in a row' : 'days in a row'"
      />
      <StatCard
          v-if="stats.average_rating !== null"
          :icon="StarIcon"
          label="Avg. rating"
          :value="stats.average_rating.toFixed(1)"
          subtext="of finished books"
      />
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, computed, watch} from 'vue';
import {
  BookOpenIcon,
  CalendarDaysIcon,
  DocumentTextIcon,
  FireIcon,
  PlusCircleIcon,
  StarIcon,
} from '@heroicons/vue/24/outline';
import StatCard from '@/components/stats/StatCard.vue';
import {apiFetch} from '@/api/client';
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
    const stats = ref<PeriodOverview | null>(null);
    const loading = ref(true);

    const fetchStats = async () => {
      loading.value = true;
      try {
        const res = await apiFetch('/api/stats/overview', {
          method: 'POST',
          body: JSON.stringify({
            mode: props.mode,
            year: props.year,
            month: props.mode === 'month' ? props.month : undefined,
          }),
        });
        if (res.ok) {
          stats.value = await res.json();
        } else {
          stats.value = null;
        }
      } catch (e) {
        console.error('Failed to fetch overview stats:', e);
        stats.value = null;
      } finally {
        loading.value = false;
      }
    };

    const periodLabel = computed(() =>
        props.mode === 'year'
            ? `${props.year}`
            : moment().year(props.year).month(props.month - 1).format('MMM YYYY'),
    );

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

    watch(() => [props.mode, props.year, props.month], fetchStats, {immediate: true});

    return {
      stats,
      loading,
      periodLabel,
      finishedInLabel,
      pagesPerDay,
      formatNumber,
      BookOpenIcon,
      CalendarDaysIcon,
      DocumentTextIcon,
      FireIcon,
      PlusCircleIcon,
      StarIcon,
    };
  },
});
</script>
