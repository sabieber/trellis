<!-- How the books finished in the period are spread across 100-page length bands. -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t('stats.pageLengthTitle') }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 flex flex-col flex-1 justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="total === 0" class="t-meta text-center py-14">
        {{ $t('stats.noPageData') }}
      </div>
      <MiniBars v-else :bars="bars" highlight-peak/>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, type PropType} from 'vue';
import MiniBars from '@/components/stats/MiniBars.vue';
import {formatPeriod} from '@/utils/period';

export default defineComponent({
  components: {MiniBars},
  props: {
    mode: {type: String, required: true},
    year: {type: Number, required: true},
    month: {type: Number, required: true},
    distribution: {type: Array as PropType<number[]>, default: () => []},
    loading: {type: Boolean, default: false},
  },
  setup(props) {
    const bars = computed(() =>
        props.distribution.map((value, index) => ({
          label: index === 0 ? '<100' : `${index * 100}+`,
          value,
        })),
    );

    const total = computed(() => props.distribution.reduce((sum, count) => sum + count, 0));

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    return {bars, total, periodLabel};
  },
});
</script>
