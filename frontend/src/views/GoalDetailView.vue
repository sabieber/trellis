<template>
  <PageContainer :title="goalLabel" :description="periodDescription" ref="pageContainer">
    <template #title-button>
      <div v-if="!loading && books.length" class="flex items-center gap-2">
        <select v-model="sortBy" class="select w-36">
          <option value="finished_at">{{ $t('shelf.sortFinished') }}</option>
          <option value="title">{{ $t('shelf.sortTitle') }}</option>
          <option value="author">{{ $t('shelf.sortAuthor') }}</option>
        </select>
        <LayoutModeSelect v-model="layoutMode"/>
      </div>
    </template>

    <div v-if="goal" class="mb-5">
      <div class="flex justify-between t-meta mb-1.5">
        <span>{{ goal.progress }} / {{ goal.target }} {{ goal.goal_type === 'books' ? $t('common.books') : $t('common.pages') }}</span>
        <span class="text-green-soft">{{ goal.percentage }}%</span>
      </div>
      <PlainProgress :pct="goal.percentage"/>
    </div>

    <!-- Only year/month goals: the activity endpoint has no 'week' bucketing. -->
    <GoalProgressChart
        v-if="goal && (goal.timeframe === 'year' || goal.timeframe === 'month')"
        :goal="goal"
        class="mb-6"
    />

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <BookLayout
        v-else-if="books.length"
        :books="sortedBooks"
        :mode="layoutMode"
        :date-label="$t('shelf.finished')"
        date-field="finished_at"
    />

    <div v-else class="t-meta text-center py-12">{{ $t('goalDetail.noBooks') }}</div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted} from 'vue';
import {useRoute} from 'vue-router';
import {useI18n} from 'vue-i18n';
import PageContainer from '@/components/PageContainer.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import PlainProgress from '@/components/ui/PlainProgress.vue';
import GoalProgressChart from '@/components/GoalProgressChart.vue';
import {apiFetch} from '@/api/client';
import {useLayoutMode} from '@/composables/useLayoutMode';
import moment from 'moment';
import type {ShelfBook} from '@/types/shelf';

// Its own key: this view lists a different set of books than the shelf, author
// and browse views, which share one preference.
const LAYOUT_STORAGE_KEY = 'goal-detail-layout-mode';

interface GoalDetail {
  id: string;
  goal_type: string;
  timeframe: string;
  target: number;
  progress: number;
  percentage: number;
  period_start: string;
  period_end: string;
}

interface GoalBook extends ShelfBook {
  finished_at: string | null;
  total_pages: number;
}

export default defineComponent({
  components: {
    PageContainer, BookLayout, LayoutModeSelect,
    PlainProgress, GoalProgressChart,
  },
  setup() {
    const {t, locale} = useI18n();
    const route = useRoute();
    const goal = ref<GoalDetail | null>(null);
    const books = ref<GoalBook[]>([]);
    const loading = ref(true);
    const sortBy = ref<'finished_at' | 'title' | 'author'>('finished_at');
    const pageContainer = ref<any>(null);
    const layoutMode = useLayoutMode(LAYOUT_STORAGE_KEY);

    const goalLabel = computed(() => {
      if (!goal.value) return '';
      const g = goal.value;
      const start = new Date(g.period_start + 'T00:00:00');
      const end = new Date(g.period_end + 'T00:00:00');
      const type = g.goal_type === 'books' ? t('common.books') : t('common.pages');
      const month = start.toLocaleDateString(locale.value, {month: 'short'});
      if (g.timeframe === 'year') return t('home.goalYear', {type, year: start.getFullYear()});
      if (g.timeframe === 'month') return t('home.goalMonth', {type, month, year: start.getFullYear()});
      return t('home.goalWeek', {type, month, from: start.getDate(), to: end.getDate()});
    });

    const periodDescription = computed(() => {
      if (!goal.value) return '';
      return t('goals.periodRange', {
        from: moment(goal.value.period_start).format('MMM D'),
        to: moment(goal.value.period_end).format('MMM D, YYYY'),
      });
    });

    const sortedBooks = computed(() => {
      const arr = [...books.value];
      if (sortBy.value === 'title') {
        arr.sort((a, b) => (a.title || '').localeCompare(b.title || ''));
      } else if (sortBy.value === 'author') {
        arr.sort((a, b) => (a.author || '').localeCompare(b.author || ''));
      } else {
        arr.sort((a, b) => (b.finished_at || '').localeCompare(a.finished_at || ''));
      }
      return arr;
    });

    const fetchGoalDetail = async (goalId: string) => {
      try {
        const response = await apiFetch('/api/goals/detail', {
          method: 'POST',
          body: JSON.stringify({goal_id: goalId}),
        });
        if (response.ok) {
          const data = await response.json();
          goal.value = {
            id: data.id,
            goal_type: data.goal_type,
            timeframe: data.timeframe,
            target: data.target,
            progress: data.progress,
            percentage: data.percentage,
            period_start: data.period_start,
            period_end: data.period_end,
          };
          books.value = data.contributing_books;
        }
      } catch (error) {
        console.error('Failed to fetch goal detail:', error);
      } finally {
        loading.value = false;
      }
    };

    onMounted(() => {
      fetchGoalDetail(route.params.id as string);
    });

    return {
      goal, books, loading, sortBy,
      goalLabel, periodDescription, sortedBooks, layoutMode,
      pageContainer,
    };
  },
});
</script>
