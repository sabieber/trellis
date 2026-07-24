<template>
  <PageContainer title="Statistics" description="Your reading at a glance" wide>
    <div class="flex flex-wrap items-center justify-between gap-3 mb-7">
      <SegmentedControl
          :model-value="mode"
          @update:model-value="setMode"
          :options="[{value: 'year', label: 'Year'}, {value: 'month', label: 'Month'}]"
          class="w-44"
      />
      <div class="flex items-center gap-1.5">
        <button
            class="flex items-center justify-center size-8 rounded-full text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
            aria-label="Previous period"
            @click="step(-1)"
        >
          <ChevronLeftIcon class="size-4.5"/>
        </button>
        <span class="t-title text-base min-w-24 text-center select-none">{{ stepperLabel }}</span>
        <button
            class="flex items-center justify-center size-8 rounded-full text-muted transition-colors duration-150"
            :class="atCurrentPeriod ? 'opacity-40 cursor-default' : 'cursor-pointer hover:text-ink hover:bg-surface-2'"
            aria-label="Next period"
            :disabled="atCurrentPeriod"
            @click="step(1)"
        >
          <ChevronRightIcon class="size-4.5"/>
        </button>
      </div>
    </div>

    <div class="flex flex-col gap-7">
      <PeriodOverviewSection :mode="mode" :year="year" :month="month"/>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed} from 'vue';
import {ChevronLeftIcon, ChevronRightIcon} from '@heroicons/vue/24/outline';
import PageContainer from '@/components/PageContainer.vue';
import PeriodOverviewSection from '@/components/stats/PeriodOverviewSection.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import moment from 'moment';

export default defineComponent({
  components: {PageContainer, PeriodOverviewSection, SegmentedControl, ChevronLeftIcon, ChevronRightIcon},
  setup() {
    const now = new Date();
    const currentYear = now.getFullYear();
    const currentMonth = now.getMonth() + 1;

    const mode = ref<'year' | 'month'>('year');
    const year = ref(currentYear);
    const month = ref(currentMonth);

    const setMode = (value: string) => {
      mode.value = value as 'year' | 'month';
      if (mode.value === 'month') {
        month.value = year.value === currentYear ? currentMonth : 1;
      }
    };

    const atCurrentPeriod = computed(() =>
        mode.value === 'year'
            ? year.value >= currentYear
            : year.value > currentYear || (year.value === currentYear && month.value >= currentMonth),
    );

    const step = (direction: number) => {
      if (direction > 0 && atCurrentPeriod.value) return;
      if (mode.value === 'year') {
        year.value += direction;
      } else {
        const date = new Date(year.value, month.value - 1 + direction, 1);
        year.value = date.getFullYear();
        month.value = date.getMonth() + 1;
      }
    };

    const stepperLabel = computed(() =>
        mode.value === 'year'
            ? `${year.value}`
            : moment().year(year.value).month(month.value - 1).format('MMM YYYY'),
    );

    return {mode, year, month, setMode, step, stepperLabel, atCurrentPeriod};
  },
});
</script>
