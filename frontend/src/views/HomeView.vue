<template>
  <div class="min-h-screen bg-base-300 flex flex-col">
    <div v-if="auth.isAuthenticated" class="flex flex-col gap-6 px-4 pt-5 pb-4">

      <!-- Goals section -->
      <div>
        <div class="flex justify-between items-center mb-3">
          <h2 class="font-bold text-lg">Reading Goals</h2>
          <RouterLink to="/goals" class="text-sm text-primary flex items-center gap-0.5">
            See all <ChevronRightIcon class="size-4" />
          </RouterLink>
        </div>

        <div v-if="goalsLoading" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-sm"></span>
        </div>
        <div v-else-if="goals.length === 0" class="text-sm opacity-50 text-center py-4">
          No goals yet.
          <RouterLink to="/goals" class="text-primary ml-1">Create one</RouterLink>
        </div>
        <div v-else class="flex flex-col gap-2">
          <div v-for="goal in goals.slice(0, 3)" :key="goal.id" class="bg-base-200 rounded-xl p-3">
            <div class="flex justify-between items-center mb-1">
              <span class="font-medium text-sm">{{ formatGoalLabel(goal) }}</span>
              <span class="text-xs" :class="goal.percentage >= 100 ? 'text-success' : 'opacity-50'">
                {{ goal.progress }}/{{ goal.target }}
              </span>
            </div>
            <progress
              class="progress w-full h-1.5"
              :class="goal.percentage >= 100 ? 'progress-success' : 'progress-primary'"
              :value="goal.progress"
              :max="goal.target"
            ></progress>
          </div>
        </div>
      </div>

      <!-- Currently Reading section -->
      <div>
        <div class="flex justify-between items-center mb-3">
          <h2 class="font-bold text-lg">Currently Reading</h2>
          <RouterLink to="/library" class="text-sm text-primary flex items-center gap-0.5">
            See all <ChevronRightIcon class="size-4" />
          </RouterLink>
        </div>

        <div v-if="readingsLoading" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-sm"></span>
        </div>
        <div v-else-if="activeReadings.length === 0" class="text-sm opacity-50 text-center py-4">
          Nothing in progress yet.
        </div>
        <div v-else class="flex flex-col gap-2">
          <div v-for="reading in activeReadings" :key="reading.reading_id" class="bg-base-200 rounded-xl p-3 flex gap-3 items-center">
            <!-- Cover -->
            <div
              class="w-12 h-16 rounded-lg flex-none overflow-hidden relative"
              :style="{ backgroundColor: getBookColor(reading.book_id) }"
            >
              <img
                v-if="reading.google_books_id"
                :src="`https://books.google.com/books/content?id=${reading.google_books_id}&printsec=frontcover&img=1&zoom=1&source=gbs_api`"
                class="absolute inset-0 w-full h-full object-cover"
                loading="lazy"
              />
            </div>

            <!-- Info -->
            <div class="flex-1 min-w-0">
              <p class="font-medium text-sm leading-tight truncate">{{ reading.title || 'Untitled' }}</p>
              <p class="text-xs opacity-50 truncate mb-1">{{ reading.author || 'Unknown author' }}</p>
              <p class="text-xs opacity-50 mb-1">Page {{ reading.progress }} of {{ reading.total_pages }} · {{ readingPercent(reading) }}%</p>
              <progress
                class="progress progress-primary w-full h-1.5"
                :value="reading.progress"
                :max="reading.total_pages"
              ></progress>
            </div>

            <!-- Update button -->
            <button
              @click="openUpdateModal(reading)"
              class="btn btn-sm btn-neutral rounded-full flex-none"
            >
              Update
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="flex flex-col items-center justify-center flex-1 px-4">
      <h2 class="text-2xl font-semibold text-center">Welcome to trellis</h2>
      <p class="opacity-50 text-center mt-2">Track your reading, set goals, and build your library.</p>
    </div>

    <!-- Track progress modal -->
    <TrackProgressModal
      v-if="updateTarget"
      :initialProgress="updateTarget.progress"
      @close="updateTarget = null"
      @submit="submitProgress"
    />

    <!-- Toast -->
    <div v-if="toastMessage" class="toast toast-top toast-center pt-16">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import { RouterLink } from 'vue-router';
import { ChevronRightIcon } from '@heroicons/vue/16/solid';
import { useAuthStore } from '@/stores/auth';
import TrackProgressModal from '@/components/TrackProgressModal.vue';
import { apiFetch } from '@/api/client';

interface Goal {
  id: string;
  goal_type: string;
  timeframe: string;
  target: number;
  progress: number;
  percentage: number;
  period_start: string;
  period_end: string;
}

interface ActiveReading {
  reading_id: string;
  book_id: string;
  title: string | null;
  author: string | null;
  google_books_id: string | null;
  progress: number;
  total_pages: number;
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

const BOOK_COLORS = [
  '#1e3a5f', '#4a2500', '#2d1454', '#3a0f0f', '#0f3d2a',
  '#1a1a4e', '#3d2b00', '#1f3a2a', '#3a1a00', '#0f2a3d',
];

function getBookColor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = ((hash << 5) - hash) + id.charCodeAt(i);
    hash |= 0;
  }
  return BOOK_COLORS[Math.abs(hash) % BOOK_COLORS.length];
}

export default defineComponent({
  components: { RouterLink, ChevronRightIcon, TrackProgressModal },
  setup() {
    const auth = useAuthStore();
    const goals = ref<Goal[]>([]);
    const goalsLoading = ref(false);
    const activeReadings = ref<ActiveReading[]>([]);
    const readingsLoading = ref(false);
    const updateTarget = ref<ActiveReading | null>(null);
    const toastMessage = ref('');
    const toastType = ref('');

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => { toastMessage.value = ''; toastType.value = ''; }, 3000);
    };

    const fetchGoals = async () => {
      if (!auth.isAuthenticated) return;
      goalsLoading.value = true;
      try {
        const res = await apiFetch('/api/goals/list', { method: 'POST' });
        if (res.ok) {
          const data = await res.json();
          goals.value = data.goals;
        }
      } catch (e) {
        console.error('Failed to fetch goals:', e);
      } finally {
        goalsLoading.value = false;
      }
    };

    const fetchActiveReadings = async () => {
      if (!auth.isAuthenticated) return;
      readingsLoading.value = true;
      try {
        const res = await apiFetch('/api/readings/active', { method: 'POST' });
        if (res.ok) {
          const data = await res.json();
          activeReadings.value = data.readings;
        }
      } catch (e) {
        console.error('Failed to fetch active readings:', e);
      } finally {
        readingsLoading.value = false;
      }
    };

    const openUpdateModal = (reading: ActiveReading) => {
      updateTarget.value = reading;
    };

    const submitProgress = async (progress: number, readAt: string) => {
      if (!updateTarget.value) return;
      try {
        const res = await apiFetch('/api/books/track-progress', {
          method: 'POST',
          body: JSON.stringify({ reading_id: updateTarget.value.reading_id, progress, read_at: readAt }),
        });
        if (res.ok) {
          updateTarget.value = null;
          showToast('Progress updated!', 'alert-success');
          await fetchActiveReadings();
        } else {
          const err = await res.json();
          showToast(err.error || 'Failed to update progress.', 'alert-error');
        }
      } catch {
        showToast('Failed to update progress.', 'alert-error');
      }
    };

    const readingPercent = (r: ActiveReading) =>
      r.total_pages > 0 ? Math.round((r.progress / r.total_pages) * 100) : 0;

    const formatGoalLabel = (goal: Goal): string => {
      const start = new Date(goal.period_start + 'T00:00:00');
      const end = new Date(goal.period_end + 'T00:00:00');
      const typeLabel = goal.goal_type === 'books' ? 'Books' : 'Pages';
      if (goal.timeframe === 'year') return `${typeLabel} in ${start.getFullYear()}`;
      if (goal.timeframe === 'month') return `${typeLabel} in ${MONTHS[start.getMonth()]} ${start.getFullYear()}`;
      return `${typeLabel} in ${MONTHS[start.getMonth()]} ${start.getDate()}–${end.getDate()}`;
    };

    onMounted(() => {
      fetchGoals();
      fetchActiveReadings();
    });

    return {
      auth, goals, goalsLoading, activeReadings, readingsLoading,
      updateTarget, toastMessage, toastType,
      openUpdateModal, submitProgress, readingPercent,
      formatGoalLabel, getBookColor,
    };
  },
});
</script>
