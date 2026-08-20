<template>
  <div class="min-h-screen flex flex-col">
    <div v-if="auth.isAuthenticated" class="flex flex-col gap-6 px-4 pt-5 pb-4">
      <div>
        <div class="t-display text-2xl">{{ greeting }}</div>
        <div class="t-meta mt-1">{{ today }}</div>
      </div>

      <!-- Reading streak section -->
      <div v-if="streak">
        <h2 class="t-eyebrow mb-3">{{ $t('home.readingStreak') }}</h2>
        <StreakBed
            :current-days="streak.current_days"
            :longest-days="streak.longest_days"
            :current-weeks="streak.current_weeks"
            :week="streak.week"
        />
      </div>

      <!-- Goals section -->
      <div>
        <div class="flex justify-between items-center mb-3">
          <h2 class="t-eyebrow">{{ $t('home.readingGoals') }}</h2>
          <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" to="/goals">
            {{ $t('common.seeAll') }}
            <ChevronRightIcon class="size-4"/>
          </Button>
        </div>

        <div v-if="goalsLoading" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-sm"></span>
        </div>
        <div v-else-if="goals.length === 0" class="t-meta text-center py-4">
          {{ $t('home.noGoals') }}
          <RouterLink to="/goals" class="text-green-soft ml-1 hover:text-green transition-colors duration-150">{{ $t('home.createOne') }}</RouterLink>
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
          <h2 class="t-eyebrow">{{ $t('home.currentlyReading') }}</h2>
          <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" to="/library">
            {{ $t('common.seeAll') }}
            <ChevronRightIcon class="size-4"/>
          </Button>
        </div>

        <!-- Only the first load gets a spinner. A refetch after logging progress
             keeps the cards mounted, so the vine can grow into its new value
             instead of being torn down and rebuilt at it. -->
        <div v-if="readingsLoading && activeReadings.length === 0" class="flex justify-center py-4">
          <span class="loading loading-spinner loading-sm"></span>
        </div>
        <div v-else-if="activeReadings.length === 0" class="t-meta text-center py-4">
          {{ $t('home.nothingInProgress') }}
        </div>
        <div v-else class="flex flex-col gap-2.5">
          <div
              v-for="reading in activeReadings"
              :key="reading.reading_id"
              class="relative bg-surface border border-line rounded-md p-3.5 flex gap-3.5 hoverable-card"
          >
            <div class="flex gap-3.5 flex-1 min-w-0">
              <BookCover
                  :title="reading.title || $t('common.untitled')"
                  :author="reading.author || ''"
                  :width="64"
                  :cover-url="resolvedCoverUrl(reading.book_id, bookCoverUrl(reading))"
                  :book-id="reading.book_id"
                  :has-note="reading.has_notes"
                  @resolve-cover="onResolveCover"
              />
              <div class="flex-1 min-w-0 flex flex-col justify-between">
                <div>
                  <p class="t-title text-base leading-tight truncate">
                    <RouterLink
                        class="stretched-link"
                        :to="{ name: 'book-detail', params: { id: reading.book_id }, query: { tab: 'Log' } }"
                    >{{ reading.title || $t('common.untitled') }}</RouterLink>
                  </p>
                  <div class="flex items-center gap-1.5 mt-0.5 min-w-0">
                    <p class="t-meta truncate">
                      <RouterLink
                          v-if="isLinkableAuthor(reading.author)"
                          class="relative z-1 hover:text-green-soft hover:underline transition-colors duration-150"
                          :to="authorRoute(reading.author)"
                      >{{ reading.author }}</RouterLink>
                      <span v-else>{{ $t('common.unknownAuthor') }}</span>
                    </p>
                    <span class="badge badge-sm flex-none">{{ reading.mode === 'percentage' ? $t('readingModal.modePercentage') : $t('readingModal.modePages') }}</span>
                  </div>
                </div>
                <div>
                  <VineProgress :pct="readingPercent(reading)" :height="18"/>
                  <p class="t-meta mt-1.5">
                    <template v-if="reading.mode !== 'percentage'">
                      {{ $t('home.pageOf', { current: reading.progress, total: reading.total_pages }) }} ·
                    </template>
                    <span class="text-green-soft">{{ readingPercent(reading) }}%</span>
                  </p>
                </div>
              </div>
            </div>
            <Button variant="soft" class="relative z-1 self-center flex-none px-3.5! py-2! text-[13px]!"
                    @click="openUpdateModal(reading)">
              {{ $t('common.update') }}
            </Button>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="lattice-bg flex flex-col items-center justify-center flex-1 px-4">
      <img src="/logo.svg" class="size-20 rounded-full shadow-soft" alt=""/>
      <h2 class="t-display text-4xl mt-4">trellis</h2>
      <p class="t-italic text-green-soft text-lg mt-1">{{ $t('auth.tagline') }}</p>
      <p class="t-meta text-center mt-3.5">{{ $t('home.growTagline') }}</p>
    </div>

    <!-- Track progress modal -->
    <TrackProgressModal
        v-if="updateTarget"
        :initialProgress="updateTarget.progress"
        :totalPages="updateTarget.total_pages"
        :mode="updateTarget.mode"
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
import {computed, defineComponent, ref, onMounted} from 'vue';
import {RouterLink, useRouter} from 'vue-router';
import {useI18n} from 'vue-i18n';
import {ChevronRightIcon} from '@lucide/vue';
import {useAuthStore} from '@/stores/auth';
import {apiErrorMessage} from '@/utils/apiError';
import TrackProgressModal from '@/components/TrackProgressModal.vue';
import Button from '@/components/ui/Button.vue';
import BookCover from '@/components/ui/BookCover.vue';
import VineProgress from '@/components/ui/VineProgress.vue';
import StreakBed, {type StreakDay} from '@/components/StreakBed.vue';
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

interface Streak {
  current_days: number;
  longest_days: number;
  current_weeks: number;
  longest_weeks: number;
  week: StreakDay[];
}

interface ActiveReading {
  reading_id: string;
  book_id: string;
  title: string | null;
  author: string | null;
  google_books_id: string | null;
  isbn13: string | null;
  isbn10: string | null;
  cover_url: string | null;
  progress: number;
  total_pages: number;
  mode: string;
  has_notes: boolean;
}

import {bookCoverUrl} from '@/utils/coverUrl';
import {authorRoute, isLinkableAuthor} from '@/utils/authorRoute';
import {useBookCovers} from '@/composables/useBookCovers';

export default defineComponent({
  components: {RouterLink, ChevronRightIcon, TrackProgressModal, Button, PlainProgress, BookCover, VineProgress, StreakBed},
  setup() {
    const {t, locale} = useI18n();
    const auth = useAuthStore();
    const goals = ref<Goal[]>([]);
    const goalsLoading = ref(false);
    const streak = ref<Streak | null>(null);
    const activeReadings = ref<ActiveReading[]>([]);
    const readingsLoading = ref(false);
    const updateTarget = ref<ActiveReading | null>(null);
    const toastMessage = ref('');
    const toastType = ref('');

    const now = new Date();
    const hour = now.getHours();
    const daypart = hour < 12 ? 'morning' : hour < 18 ? 'afternoon' : 'evening';
    const greeting = computed(() => t(`home.greeting.${daypart}`));
    const today = computed(() => now.toLocaleDateString(locale.value, {weekday: 'long', day: 'numeric', month: 'long'}));

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

    const fetchStreak = async () => {
      if (!auth.isAuthenticated) return;
      try {
        const res = await apiFetch('/api/stats/streak', {method: 'POST'});
        if (res.ok) {
          streak.value = await res.json();
        }
      } catch (e) {
        console.error('Failed to fetch streak:', e);
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
          showToast(t('home.progressUpdated'), 'alert-success');
          // Logging progress can plant today's seedling, so the bed reloads too.
          await Promise.all([fetchActiveReadings(), fetchStreak()]);
        } else {
          showToast(apiErrorMessage(res.status, t), 'alert-error');
        }
      } catch {
        showToast(t('error.network'), 'alert-error');
      }
    };

    const readingPercent = (r: ActiveReading) =>
        r.total_pages > 0 ? Math.round((r.progress / r.total_pages) * 100) : 0;

    const formatGoalLabel = (goal: Goal): string => {
      const start = new Date(goal.period_start + 'T00:00:00');
      const end = new Date(goal.period_end + 'T00:00:00');
      const type = goal.goal_type === 'books' ? t('common.books') : t('common.pages');
      const month = start.toLocaleDateString(locale.value, {month: 'short'});
      if (goal.timeframe === 'year') return t('home.goalYear', {type, year: start.getFullYear()});
      if (goal.timeframe === 'month') return t('home.goalMonth', {type, month, year: start.getFullYear()});
      if (goal.timeframe === 'day') return t('home.goalDay', {type});
      return t('home.goalWeek', {type, month, from: start.getDate(), to: end.getDate()});
    };

    const { resolvedCoverUrl, onResolveCover } = useBookCovers();

    onMounted(() => {
      fetchGoals();
      fetchStreak();
      fetchActiveReadings();
    });

    return {
      auth, goals, goalsLoading, streak, activeReadings, readingsLoading,
      updateTarget, toastMessage, toastType,
      openUpdateModal, submitProgress, readingPercent,
      formatGoalLabel, bookCoverUrl, resolvedCoverUrl, onResolveCover, greeting, today,
      authorRoute, isLinkableAuthor,
    };
  },
});
</script>
