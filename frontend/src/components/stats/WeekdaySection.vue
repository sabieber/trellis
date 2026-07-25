<!-- Pages logged per weekday across the period, to surface which days are the
     most productive reading days. -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">Reading by weekday</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
      <span v-if="!loading && total > 0" class="t-meta">busiest on {{ peakLabel }}</span>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 flex flex-col flex-1 justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="total === 0" class="t-meta text-center py-14">
        No pages logged in this period.
      </div>
      <MiniBars v-else :bars="bars" highlight-peak/>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, type PropType} from 'vue';
import MiniBars from '@/components/stats/MiniBars.vue';
import {formatPeriod} from '@/utils/period';

const LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

export default defineComponent({
  components: {MiniBars},
  props: {
    mode: {type: String, required: true},
    year: {type: Number, required: true},
    month: {type: Number, required: true},
    weekdayPages: {type: Array as PropType<number[]>, default: () => []},
    loading: {type: Boolean, default: false},
  },
  setup(props) {
    const bars = computed(() =>
        LABELS.map((label, index) => ({label, value: props.weekdayPages[index] ?? 0})),
    );

    const total = computed(() => props.weekdayPages.reduce((sum, pages) => sum + pages, 0));

    const peakLabel = computed(() => {
      let peak = 0;
      props.weekdayPages.forEach((pages, index) => {
        if (pages > (props.weekdayPages[peak] ?? 0)) peak = index;
      });
      return LABELS[peak];
    });

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    return {bars, total, peakLabel, periodLabel};
  },
});
</script>
