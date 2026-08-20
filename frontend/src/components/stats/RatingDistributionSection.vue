<!-- How the books finished in the period are spread across the rating scores:
     five bars in star mode, two tendency bars in thumbs mode. -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t('stats.ratingsTitle') }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
      <span v-if="!loading && rated > 0" class="t-meta">{{ summary }}</span>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 flex flex-col flex-1 justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="rated === 0" class="t-meta text-center py-14">
        {{ $t('stats.noRated') }}
      </div>
      <MiniBars v-else :bars="bars"/>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, type PropType} from 'vue';
import {useI18n} from 'vue-i18n';
import MiniBars from '@/components/stats/MiniBars.vue';
import {ratingMode} from '@/utils/ratingMode';
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
    const {t} = useI18n();

    const count = (score: number) => props.distribution[score - 1] ?? 0;

    // Thumbs mode collapses the five scores into the three thumbs. The 3s keep
    // their own bar: they were rated, but they lean nowhere, so counting them as
    // a like or a dislike would invent an opinion.
    const bars = computed(() => {
      if (ratingMode.value === 'thumbs') {
        return [
          {label: t('stats.disliked'), value: count(1) + count(2)},
          {label: t('stats.soso'), value: count(3)},
          {label: t('stats.liked'), value: count(4) + count(5)},
        ];
      }
      return [1, 2, 3, 4, 5].map((score) => ({label: score, value: count(score)}));
    });

    const rated = computed(() => props.distribution.reduce((sum, count) => sum + count, 0));

    const average = computed(() => {
      if (rated.value === 0) return '0.0';
      const weighted = props.distribution.reduce((sum, count, index) => sum + count * (index + 1), 0);
      return (weighted / rated.value).toFixed(1);
    });

    // A mean of 1s and 5s says nothing, so thumbs mode reports the share of the
    // books with a tendency that got a thumbs up.
    const summary = computed(() => {
      if (ratingMode.value !== 'thumbs') {
        return t('stats.ratedAvg', {count: rated.value, avg: average.value});
      }
      const up = count(4) + count(5);
      const withTendency = up + count(1) + count(2);
      if (withTendency === 0) return t('stats.ratedCount', {count: rated.value});
      return t('stats.ratedLiked', {
        count: rated.value,
        percent: Math.round((up / withTendency) * 100),
      });
    });

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    return {bars, rated, summary, periodLabel};
  },
});
</script>
