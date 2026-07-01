<template>
  <PageContainer :title="goalLabel" :description="periodDescription" wide ref="pageContainer">
    <template #title-button>
      <div v-if="!loading && books.length" class="flex items-center gap-2">
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="finished_at">Finished at</option>
          <option value="title">Book name</option>
          <option value="author">Author name</option>
        </select>
        <SegmentedControl v-model="layoutMode" :options="layoutOptions">
          <template #option="{ option }">
            <QueueListIcon v-if="option.value === 'list'" class="size-4"/>
            <Squares2X2Icon v-else-if="option.value === 'grid'" class="size-4"/>
            <BookOpenIcon v-else class="size-4"/>
          </template>
        </SegmentedControl>
      </div>
    </template>

    <div v-if="goal" class="mb-5">
      <div class="flex justify-between t-meta mb-1.5">
        <span>{{ goal.progress }} / {{ goal.target }} {{ goal.goal_type === 'books' ? 'books' : 'pages' }}</span>
        <span class="text-green-soft">{{ goal.percentage }}%</span>
      </div>
      <PlainProgress :pct="goal.percentage"/>
    </div>

    <div ref="contentRef">
      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <template v-else-if="books.length">
        <ShelfListView
            v-if="layoutMode === 'list'"
            :books="sortedBooks"
            :cover-width="listCoverWidth"
            dateLabel="Finished"
            dateField="finished_at"
            @view-book="viewBookDetail"
            @remove-book="() => {}"
        />
        <ShelfGridView
            v-else-if="layoutMode === 'grid'"
            :books="sortedBooks"
            :tile-width="gridTileWidth"
            @view-book="viewBookDetail"
        />
        <ShelfBoardView
            v-else
            :books="sortedBooks"
            :spine-height="spineHeight"
            :container-width="containerWidth"
            @view-book="viewBookDetail"
        />
      </template>

      <div v-else class="t-meta text-center py-12">No books finished in this period yet.</div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {QueueListIcon, Squares2X2Icon, BookOpenIcon} from '@heroicons/vue/24/outline';
import PageContainer from '@/components/PageContainer.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import ShelfListView from '@/components/shelf/ShelfListView.vue';
import ShelfGridView from '@/components/shelf/ShelfGridView.vue';
import ShelfBoardView from '@/components/shelf/ShelfBoardView.vue';
import PlainProgress from '@/components/ui/PlainProgress.vue';
import {apiFetch} from '@/api/client';
import {useContainerWidth} from '@/composables/useContainerWidth';
import moment from 'moment';
import type {ShelfBook} from '@/types/shelf';

const LAYOUT_STORAGE_KEY = 'goal-detail-layout-mode';
const MD_BREAKPOINT = 768;
const GRID_TILE_SM = 80;
const GRID_TILE_LG = 112;
const SPINE_HEIGHT_SM = 160;
const SPINE_HEIGHT_LG = 200;
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

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
    QueueListIcon, Squares2X2Icon, BookOpenIcon,
    PageContainer, SegmentedControl,
    ShelfListView, ShelfGridView, ShelfBoardView,
    PlainProgress,
  },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const goal = ref<GoalDetail | null>(null);
    const books = ref<GoalBook[]>([]);
    const loading = ref(true);
    const sortBy = ref<'finished_at' | 'title' | 'author'>('finished_at');
    const pageContainer = ref<any>(null);
    const contentRef = ref<HTMLElement | null>(null);

    const layoutOptions = [
      {value: 'list'},
      {value: 'grid'},
      {value: 'shelf'},
    ];

    const validLayouts = ['list', 'grid', 'shelf'];
    const saved = localStorage.getItem(LAYOUT_STORAGE_KEY);
    const layoutMode = ref(validLayouts.includes(saved || '') ? saved! : 'list');
    watch(layoutMode, (val) => localStorage.setItem(LAYOUT_STORAGE_KEY, val));

    const isReady = computed(() => !loading.value);
    const {containerWidth} = useContainerWidth(contentRef, isReady);

    const listCoverWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? 72 : 56
    );

    const gridTileWidth = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? GRID_TILE_LG : GRID_TILE_SM
    );

    const spineHeight = computed(() =>
        containerWidth.value >= MD_BREAKPOINT ? SPINE_HEIGHT_LG : SPINE_HEIGHT_SM
    );

    const goalLabel = computed(() => {
      if (!goal.value) return '';
      const g = goal.value;
      const start = new Date(g.period_start + 'T00:00:00');
      const end = new Date(g.period_end + 'T00:00:00');
      const typeLabel = g.goal_type === 'books' ? 'Books' : 'Pages';
      if (g.timeframe === 'year') return `${typeLabel} in ${start.getFullYear()}`;
      if (g.timeframe === 'month') return `${typeLabel} in ${MONTHS[start.getMonth()]} ${start.getFullYear()}`;
      return `${typeLabel} in ${MONTHS[start.getMonth()]} ${start.getDate()}\u2013${end.getDate()}`;
    });

    const periodDescription = computed(() => {
      if (!goal.value) return '';
      return `${moment(goal.value.period_start).format('MMM D')} to ${moment(goal.value.period_end).format('MMM D, YYYY')}`;
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

    const viewBookDetail = (id: string) => {
      router.push({name: 'book-detail', params: {id}});
    };

    onMounted(() => {
      fetchGoalDetail(route.params.id as string);
    });

    return {
      goal, books, loading, sortBy,
      goalLabel, periodDescription, sortedBooks,
      layoutMode, layoutOptions,
      listCoverWidth, gridTileWidth, spineHeight, containerWidth,
      pageContainer, contentRef, viewBookDetail,
    };
  },
});
</script>
