<template>
  <div class="min-h-screen flex flex-col">
    <div v-if="auth.isAuthenticated" class="flex flex-col gap-6 px-4 pt-5 pb-4">
      <div>
        <div class="t-display text-2xl">Good {{ daypart }}</div>
        <div class="t-meta mt-1">{{ today }}</div>
      </div>

      <!-- Goals section -->
      <div>
        <div class="flex justify-between items-center mb-3">
          <h2 class="t-eyebrow">Reading goals</h2>
          <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" @click="navigateTo('/goals')">
            See all
            <ChevronRightIcon class="size-4"/>
          </Button>
        </div>

        <div v-if="goalsLoading" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-sm"></span>
        </div>
        <div v-else-if="goals.length === 0" class="t-meta text-center py-4">
          No goals yet.
          <RouterLink to="/goals" class="text-green-soft ml-1 hover:text-green transition-colors duration-150">Create one</RouterLink>
        </div>
        <div v-else class="flex flex-col gap-2.5">
          <RouterLink
              v-for="goal in goals.slice(0, 3)"
              :key="goal.id"
              :to="goal.goal_type === 'books' ? { name: 'goal-detail', params: { id: goal.id } } : '/goals'"
              class="bg-surface border border-line rounded-md p-4 block hoverable-card"
          >
            <div class="flex justify-between items-center mb-2">
              <span class="font-semibold text-sm">{{ formatGoalLabel(goal) }}</span>
              <span class="t-meta" :class="{ 'text-green-soft': goal.percentage >= 100 }">
                {{ goal.progress }}/{{ goal.target }}
              </span>
            </div>
            <PlainProgress :pct="goal.percentage"/>
          </RouterLink>
        </div>
      </div>

      <!-- Currently Reading section -->
      <div>
        <div class="flex justify-between items-center mb-3">
          <h2 class="t-eyebrow">Currently reading</h2>
          <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" @click="navigateTo('/library')">
            See all
            <ChevronRightIcon class="size-4"/>
          </Button>
        </div>

        <div v-if="readingsLoading" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-sm"></span>
        </div>
        <div v-else-if="activeReadings.length === 0" class="t-meta text-center py-4">
          Nothing in progress yet.
        </div>
        <div v-else class="flex flex-col gap-2.5">
          <div
              v-for="reading in activeReadings"
              :key="reading.reading_id"
              class="bg-surface border border-line rounded-md p-3.5 flex gap-3.5 hoverable-card"
          >
            <RouterLink
                :to="{ name: 'book-detail', params: { id: reading.book_id }, query: { tab: 'Log' } }"
                class="flex gap-3.5 flex-1 min-w-0"
            >
              <BookCover
                  :title="reading.title || 'Untitled'"
                  :author="reading.author || ''"
                  :width="64"
                  :cover-url="bookCoverUrl(reading)"
              />
              <div class="flex-1 min-w-0 flex flex-col justify-between">
                <div>
                  <p class="t-title text-base leading-tight truncate">{{ reading.title || 'Untitled' }}</p>
                  <p class="t-meta truncate mt-0.5">{{ reading.author || 'Unknown author' }}</p>
                </div>
                <div>
                  <VineProgress :pct="readingPercent(reading)" :height="18"/>
                  <p class="t-meta mt-1.5">
                    page {{ reading.progress }} of {{ reading.total_pages }} ·
                    <span class="text-green-soft">{{ readingPercent(reading) }}%</span>
                  </p>
                </div>
              </div>
            </RouterLink>
            <Button variant="soft" class="self-center flex-none px-3.5! py-2! text-[13px]!"
                    @click="openUpdateModal(reading)">
              Update
            </Button>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="lattice-bg flex flex-col items-center justify-center flex-1 px-4">
      <img src="/logo.svg" class="size-20 rounded-full shadow-soft" alt=""/>
      <h2 class="t-display text-4xl mt-4">trellis</h2>
      <p class="t-italic text-green-soft text-lg mt-1">a garden library</p>
      <p class="t-meta text-center mt-3.5">grow your library and reading goals</p>
    </div>

    <!-- Track progress modal -->
    <TrackProgressModal
        v-if="updateTarget"
        :initialProgress="updateTarget.progress"
        :totalPages="updateTarget.total_pages"
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
import {defineComponent, ref, onMounted} from 'vue';
import {RouterLink, useRouter} from 'vue-router';
import {ChevronRightIcon} from '@heroicons/vue/24/outline';
import {useAuthStore} from '@/stores/auth';
import TrackProgressModal from '@/components/TrackProgressModal.vue';
import Button from '@/components/ui/Button.vue';
import BookCover from '@/components/ui/BookCover.vue';
import VineProgress from '@/components/ui/VineProgress.vue';
import {apiFetch} from '@/api/client';
import PlainProgress from "@/components/ui/PlainProgress.vue";

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
  isbn13: string | null;
  isbn10: string | null;
  progress: number;
  total_pages: number;
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

import {bookCoverUrl} from '@/utils/coverUrl';

export default defineComponent({
  components: {RouterLink, ChevronRightIcon, TrackProgressModal, Button, PlainProgress, BookCover, VineProgress},
  setup() {
    const auth = useAuthStore();
    const router = useRouter();
    const goals = ref<Goal[]>([]);
    const goalsLoading = ref(false);
    const activeReadings = ref<ActiveReading[]>([]);
    const readingsLoading = ref(false);
    const updateTarget = ref<ActiveReading | null>(null);
    const toastMessage = ref('');
    const toastType = ref('');

    const now = new Date();
    const hour = now.getHours();
    const daypart = hour < 12 ? 'morning' : hour < 18 ? 'afternoon' : 'evening';
    const today = now.toLocaleDateString(undefined, {weekday: 'long', day: 'numeric', month: 'long'});

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    const fetchGoals = async () => {
      if (!auth.isAuthenticated) return;
      goalsLoading.value = true;
      try {
        const res = await apiFetch('/api/goals/list', {method: 'POST'});
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
        const res = await apiFetch('/api/readings/active', {method: 'POST'});
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
          body: JSON.stringify({reading_id: updateTarget.value.reading_id, progress, read_at: readAt}),
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

    const navigateTo = (path: string) => router.push(path);

    onMounted(() => {
      fetchGoals();
      fetchActiveReadings();
    });

    return {
      auth, goals, goalsLoading, activeReadings, readingsLoading,
      updateTarget, toastMessage, toastType,
      openUpdateModal, submitProgress, readingPercent,
      formatGoalLabel, bookCoverUrl, daypart, today, navigateTo,
    };
  },
});
</script>
