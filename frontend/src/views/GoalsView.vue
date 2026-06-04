<template>
  <div class="min-h-screen bg-base-300 flex flex-col">
    <div class="flex justify-between items-center px-4 pt-5 pb-2">
      <h1 class="text-2xl font-bold">Reading Goals</h1>
      <button @click="showCreateModal = true" class="btn btn-sm btn-neutral rounded-full gap-1">
        <PlusIcon class="size-4" />
        New Goal
      </button>
    </div>

    <div v-if="loading" class="flex justify-center py-10">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <div v-else-if="goals.length === 0" class="text-center opacity-50 py-10">
      No goals yet. Create one to start tracking!
    </div>

    <div v-else class="flex flex-col gap-6 pb-4">
      <template v-for="section in sections" :key="section.key">
      <div v-if="section.goals.length > 0">
        <div class="flex items-baseline gap-2 px-4 mb-3">
          <h2 class="font-bold text-lg leading-tight">{{ section.label }}</h2>
          <span class="text-sm opacity-50">{{ section.goals.length }} {{ section.goals.length === 1 ? 'goal' : 'goals' }}</span>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 px-4">
          <div v-for="goal in section.goals" :key="goal.id" class="bg-base-200 rounded-xl p-4">
            <div class="flex justify-between items-start mb-1">
              <h3 class="font-semibold text-lg leading-tight">{{ formatGoalLabel(goal) }}</h3>
              <button @click="confirmDelete(goal)" class="btn btn-ghost btn-sm btn-square flex-none ml-2">
                <TrashIcon class="size-4 text-error" />
              </button>
            </div>
            <p class="text-sm opacity-50 mb-3">{{ goal.period_start }} to {{ goal.period_end }}</p>
            <div class="flex justify-between text-sm mb-1">
              <span>{{ goal.progress }} / {{ goal.target }} {{ goal.goal_type === 'books' ? 'books' : 'pages' }}</span>
              <span>{{ goal.percentage }}%</span>
            </div>
            <progress
              class="progress w-full"
              :class="goal.percentage >= 100 ? 'progress-success' : 'progress-primary'"
              :value="goal.progress"
              :max="goal.target"
            ></progress>
          </div>
        </div>
      </div>
      </template>
    </div>

    <CreateGoalModal v-if="showCreateModal" @close="showCreateModal = false" @submit="createGoal" />

    <div v-if="deleteTarget" class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">Delete Goal</h3>
        <p class="py-4">Are you sure you want to delete this {{ deleteTarget.goal_type }} goal?</p>
        <div class="modal-action">
          <button @click="doDelete" class="btn btn-error">Delete</button>
          <button @click="deleteTarget = null" class="btn btn-ghost">Cancel</button>
        </div>
      </div>
      <div class="modal-backdrop" @click="deleteTarget = null"></div>
    </div>

    <div v-if="toastMessage" class="toast toast-top toast-center pt-16">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted } from 'vue';
import CreateGoalModal from '@/components/CreateGoalModal.vue';
import { PlusIcon, TrashIcon } from '@heroicons/vue/16/solid';
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

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

export default defineComponent({
  components: { CreateGoalModal, PlusIcon, TrashIcon },
  setup() {
    const goals = ref<Goal[]>([]);
    const loading = ref(true);
    const showCreateModal = ref(false);
    const deleteTarget = ref<Goal | null>(null);
    const toastMessage = ref('');
    const toastType = ref('');

    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => { toastMessage.value = ''; toastType.value = ''; }, 3000);
    };

    const sections = computed(() => [
      { key: 'year', label: 'Yearly', goals: goals.value.filter(g => g.timeframe === 'year') },
      { key: 'month', label: 'Monthly', goals: goals.value.filter(g => g.timeframe === 'month') },
      { key: 'week', label: 'Weekly', goals: goals.value.filter(g => g.timeframe === 'week') },
    ]);

    const fetchGoals = async () => {
      loading.value = true;
      try {
        const response = await apiFetch('/api/goals/list', { method: 'POST' });
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
          showToast('Goal created!', 'alert-success');
          await fetchGoals();
        } else {
          const err = await response.json();
          showToast(err.error || 'Failed to create goal.', 'alert-error');
        }
      } catch (error) {
        console.error('Failed to create goal:', error);
        showToast('Failed to create goal.', 'alert-error');
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
          body: JSON.stringify({ goal_id: deleteTarget.value.id }),
        });
        if (response.ok) {
          deleteTarget.value = null;
          showToast('Goal deleted.', 'alert-success');
          await fetchGoals();
        } else {
          const err = await response.json();
          showToast(err.error || 'Failed to delete goal.', 'alert-error');
        }
      } catch (error) {
        console.error('Failed to delete goal:', error);
        showToast('Failed to delete goal.', 'alert-error');
      }
    };

    const formatGoalLabel = (goal: Goal): string => {
      const start = new Date(goal.period_start + 'T00:00:00');
      const end = new Date(goal.period_end + 'T00:00:00');
      const typeLabel = goal.goal_type === 'books' ? 'Books' : 'Pages';

      if (goal.timeframe === 'year') {
        return `${typeLabel} in ${start.getFullYear()}`;
      }
      if (goal.timeframe === 'month') {
        return `${typeLabel} in ${MONTHS[start.getMonth()]} ${start.getFullYear()}`;
      }
      return `${typeLabel} in ${MONTHS[start.getMonth()]} ${start.getDate()}–${end.getDate()}`;
    };

    onMounted(fetchGoals);

    return { goals, loading, showCreateModal, deleteTarget, toastMessage, toastType, sections, createGoal, confirmDelete, doDelete, formatGoalLabel };
  },
});
</script>
