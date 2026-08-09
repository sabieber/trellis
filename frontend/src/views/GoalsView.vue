<template>
  <div class="min-h-screen flex flex-col">
    <div class="flex justify-between items-center px-4 pt-5 pb-2">
      <h1 class="t-display text-2xl">{{ $t('goals.title') }}</h1>
      <Button variant="soft" class="px-3.5! py-2! text-[13px]!" @click="showCreateModal = true">
        <PlusIcon class="size-4"/>
        {{ $t('goals.newGoal') }}
      </Button>
    </div>

    <div v-if="loading" class="flex justify-center py-10">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <div v-else-if="goals.length === 0" class="t-meta text-center py-10">
      {{ $t('goals.empty') }}
    </div>

    <div v-else class="flex flex-col gap-7 pb-4 mt-3">
      <template v-for="section in sections" :key="section.key">
        <div v-if="section.goals.length > 0">
          <div class="flex items-baseline gap-2 px-4 mb-3">
            <h2 class="t-eyebrow">{{ $t(section.label) }}</h2>
            <span class="t-meta">{{ $t('goals.count', section.goals.length) }}</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 px-4">
            <div v-for="goal in section.goals" :key="goal.id" class="bg-surface border border-line rounded-md p-4 hoverable-card">
              <div class="flex justify-between items-start mb-1">
                <RouterLink
                    v-if="goal.goal_type === 'books'"
                    :to="{ name: 'goal-detail', params: { id: goal.id } }"
                    class="t-title text-base leading-tight cursor-pointer hover:text-[#7a9e7e] transition-colors duration-150 text-left"
                >
                  {{ formatGoalLabel(goal) }}
                </RouterLink>
                <h3 v-else class="t-title text-base leading-tight">{{ formatGoalLabel(goal) }}</h3>
                <button
                    @click="confirmDelete(goal)"
                    class="flex items-center justify-center size-7 rounded-full flex-none ml-2 text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
                >
                  <Trash2Icon class="size-4"/>
                </button>
              </div>
              <p class="t-meta mb-3.5">{{ formatPeriod(goal.period_start, goal.period_end) }}</p>
              <div class="flex justify-between t-meta mb-1.5">
                <span>{{ goal.progress }} / {{ goal.target }} {{
                    goal.goal_type === 'books' ? $t('common.books') : $t('common.pages')
                  }}</span>
                <span class="text-green-soft">{{ goal.percentage }}%</span>
              </div>
              <PlainProgress :pct="goal.percentage"/>
            </div>
          </div>
        </div>
      </template>
    </div>

    <CreateGoalModal v-if="showCreateModal" @close="showCreateModal = false" @submit="createGoal"/>

    <ConfirmDialog
        v-if="deleteTarget"
        :title="$t('goals.deleteTitle')"
        :message="$t('goals.deleteMessage')"
        @confirm="doDelete"
        @cancel="deleteTarget = null"
    />

    <div v-if="toastMessage" class="toast toast-top toast-center pt-16">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted} from 'vue';
import {useI18n} from 'vue-i18n';
import CreateGoalModal from '@/components/CreateGoalModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import Button from '@/components/ui/Button.vue';
import {PlusIcon, Trash2Icon} from '@lucide/vue';
import {apiFetch} from '@/api/client';
import {apiErrorMessage} from '@/utils/apiError';
import PlainProgress from "@/components/ui/PlainProgress.vue";
import moment from 'moment';

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

export default defineComponent({
  components: {PlainProgress, CreateGoalModal, ConfirmDialog, PlusIcon, Trash2Icon, Button},
  setup() {
    const {t, locale} = useI18n();
    const goals = ref<Goal[]>([]);
    const loading = ref(true);
    const showCreateModal = ref(false);
    const deleteTarget = ref<Goal | null>(null);
    const toastMessage = ref('');
    const toastType = ref('');

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    const sections = computed(() => [
      {key: 'year', label: 'goals.yearly', goals: goals.value.filter(g => g.timeframe === 'year')},
      {key: 'month', label: 'goals.monthly', goals: goals.value.filter(g => g.timeframe === 'month')},
      {key: 'week', label: 'goals.weekly', goals: goals.value.filter(g => g.timeframe === 'week')},
    ]);

    const fetchGoals = async () => {
      loading.value = true;
      try {
        const response = await apiFetch('/api/goals/list', {method: 'POST'});
        if (response.ok) {
          const data = await response.json();
          goals.value = data.goals;
        }
      } catch (error) {
        console.error('Failed to fetch goals:', error);
      } finally {
        loading.value = false;
      }
    };

    const createGoal = async (data: { goalType: string; timeframe: string; target: number }) => {
      try {
        const response = await apiFetch('/api/goals/create', {
          method: 'POST',
          body: JSON.stringify({
            goal_type: data.goalType,
            timeframe: data.timeframe,
            target: data.target,
          }),
        });
        if (response.ok) {
          showCreateModal.value = false;
          showToast(t('goals.created'), 'alert-success');
          await fetchGoals();
        } else {
          showToast(apiErrorMessage(response.status, t), 'alert-error');
        }
      } catch (error) {
        console.error('Failed to create goal:', error);
        showToast(t('error.network'), 'alert-error');
      }
    };

    const confirmDelete = (goal: Goal) => {
      deleteTarget.value = goal;
    };

    const doDelete = async () => {
      if (!deleteTarget.value) return;
      try {
        const response = await apiFetch('/api/goals/delete', {
          method: 'POST',
          body: JSON.stringify({goal_id: deleteTarget.value.id}),
        });
        if (response.ok) {
          deleteTarget.value = null;
          showToast(t('goals.deleted'), 'alert-success');
          await fetchGoals();
        } else {
          showToast(apiErrorMessage(response.status, t), 'alert-error');
        }
      } catch (error) {
        console.error('Failed to delete goal:', error);
        showToast(t('error.network'), 'alert-error');
      }
    };

    const formatPeriod = (start: string, end: string) =>
        t('goals.periodRange', {from: moment(start).format('MMM D'), to: moment(end).format('MMM D, YYYY')});

    const formatGoalLabel = (goal: Goal): string => {
      const start = new Date(goal.period_start + 'T00:00:00');
      const end = new Date(goal.period_end + 'T00:00:00');
      const type = goal.goal_type === 'books' ? t('common.books') : t('common.pages');
      const month = start.toLocaleDateString(locale.value, {month: 'short'});
      if (goal.timeframe === 'year') return t('home.goalYear', {type, year: start.getFullYear()});
      if (goal.timeframe === 'month') return t('home.goalMonth', {type, month, year: start.getFullYear()});
      return t('home.goalWeek', {type, month, from: start.getDate(), to: end.getDate()});
    };

    onMounted(fetchGoals);

    return {
      goals,
      loading,
      showCreateModal,
      deleteTarget,
      toastMessage,
      toastType,
      sections,
      createGoal,
      confirmDelete,
      doDelete,
      formatGoalLabel,
      formatPeriod,
    };
  },
});
</script>
