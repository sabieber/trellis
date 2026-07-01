<template>
  <div class="flex flex-col min-h-screen">
    <!-- Mobile top bar -->
    <div
        v-if="!$route.meta.hideNav"
        class="sm:hidden fixed w-full z-10 flex items-center gap-3 h-16 px-4 bg-bg-2/90 backdrop-blur border-b border-line"
    >
      <RouterLink to="/" class="flex items-center gap-2.5">
        <img src="/logo.svg" alt="" class="h-8 w-8 rounded-full"/>
        <span class="t-title text-xl">trellis</span>
      </RouterLink>
    </div>

    <!-- Desktop sidebar -->
    <aside
        v-if="!$route.meta.hideNav"
        class="hidden sm:flex fixed inset-y-0 left-0 z-10 w-72 flex-col bg-bg-2 border-r border-line p-5"
    >
      <RouterLink to="/" class="flex items-center gap-3 px-2 mb-8">
        <img src="/logo.svg" alt="" class="h-10 w-10 rounded-full"/>
        <span class="t-title text-[22px]">trellis</span>
      </RouterLink>
      <nav class="flex flex-col gap-1.5">
        <RouterLink
            v-for="item in navItems"
            :key="item.to"
            :to="item.to"
            class="flex items-center gap-3.5 px-4 py-3 rounded-sm text-[15px] font-semibold transition-colors duration-150"
            :class="$route.path === item.to ? 'bg-green/13 text-green' : 'text-ink-dim hover:text-ink hover:bg-surface-2'"
        >
          <component :is="item.icon" class="size-5.5"/>
          {{ item.label }}
        </RouterLink>
      </nav>
      <RouterLink
          v-if="yearGoal"
          to="/goals"
          class="mt-auto flex items-center gap-3.5 bg-surface border border-line rounded-md p-3.5 hoverable-card"
      >
        <svg width="42" height="42" viewBox="0 0 62 62" class="flex-none">
          <circle cx="31" cy="31" r="26" fill="none" stroke="#322d1e" stroke-width="6"/>
          <circle
              cx="31" cy="31" r="26" fill="none" stroke="#93c456" stroke-width="6"
              stroke-linecap="round" stroke-dasharray="163.4"
              :stroke-dashoffset="163.4 * (1 - Math.min(yearGoal.progress / yearGoal.target, 1))"
              transform="rotate(-90 31 31)"
          />
        </svg>
        <div class="min-w-0">
          <div class="t-title text-base">{{ yearGoal.progress }} / {{ yearGoal.target }}</div>
          <div class="t-meta mt-0.5">{{ currentYear }} goal</div>
        </div>
      </RouterLink>
    </aside>

    <RouterView
        class="grow"
        :class="{ 'pt-16 pb-20 sm:pt-0 sm:pb-0 sm:pl-72': !$route.meta.hideNav }"
    />

    <!-- Mobile bottom dock -->
    <nav
        v-if="!$route.meta.hideNav"
        class="sm:hidden fixed bottom-0 inset-x-0 z-10 flex justify-around items-center pt-2 pb-5 bg-bg-2 border-t border-line"
    >
      <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="flex flex-col items-center gap-1 text-[10.5px] font-semibold transition-colors duration-150"
          :class="$route.path === item.to ? 'text-green' : 'text-faint'"
      >
        <component :is="item.icon" class="size-6"/>
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, watch} from 'vue';
import {useRoute} from 'vue-router';
import {MagnifyingGlassIcon, HomeIcon, BookOpenIcon, UserIcon, ChartBarIcon} from "@heroicons/vue/24/outline";
import {useAuthStore} from '@/stores/auth';
import {apiFetch} from '@/api/client';

interface Goal {
  id: string;
  timeframe: string;
  target: number;
  progress: number;
}

export default defineComponent({
  setup() {
    const auth = useAuthStore();
    const route = useRoute();
    const yearGoal = ref<Goal | null>(null);
    const currentYear = new Date().getFullYear();

    const navItems = [
      {to: "/", label: "Home", icon: HomeIcon},
      {to: "/library", label: "Library", icon: BookOpenIcon},
      {to: "/search", label: "Search", icon: MagnifyingGlassIcon},
      {to: "/goals", label: "Goals", icon: ChartBarIcon},
      {to: "/profile", label: "Profile", icon: UserIcon},
    ];

    const fetchYearGoal = async () => {
      if (!auth.isAuthenticated) {
        yearGoal.value = null;
        return;
      }
      try {
        const res = await apiFetch('/api/goals/list', {method: 'POST'});
        if (res.ok) {
          const data = await res.json();
          yearGoal.value = data.goals.find((g: Goal) => g.timeframe === 'year') ?? null;
        }
      } catch {
        yearGoal.value = null;
      }
    };

    // Refetch on navigation so logged progress shows up without a reload.
    watch([() => auth.isAuthenticated, () => route.path], fetchYearGoal, {immediate: true});

    return {navItems, yearGoal, currentYear};
  },
});
</script>
