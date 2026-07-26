<!-- The most-read authors of the period, ranked by finished books with a
     proportional bar and the summed pages as context. -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t('stats.topAuthors') }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
      <span v-if="!loading && authors.length > 0" class="t-meta">{{ $t('stats.byBooksFinished') }}</span>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 flex-1 flex flex-col justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="authors.length === 0" class="t-meta text-center py-14">
        {{ $t('stats.noFinished') }}
      </div>

      <ol v-else class="space-y-3">
        <li v-for="(author, index) in authors" :key="author.author" class="flex items-center gap-3">
          <span class="stat-mono text-muted w-4 flex-none text-right">{{ index + 1 }}</span>
          <div class="flex-1 min-w-0">
            <div class="flex items-baseline justify-between gap-2">
              <span
                  v-if="isKnown(author.author)"
                  class="t-title text-sm truncate hover:text-green-soft hover:underline transition-colors duration-150 cursor-pointer"
                  @click="viewAuthor(author.author)"
              >{{ author.author }}</span>
              <span v-else class="t-title text-sm truncate">{{ author.author }}</span>
              <span class="t-meta flex-none">{{ $t('stats.pagesShort', { n: author.pages.toLocaleString() }) }}</span>
            </div>
            <div class="h-1.5 bg-surface-3 rounded-full mt-1.5 overflow-hidden">
              <div class="h-full bg-green rounded-full" :style="{width: `${barWidth(author.books)}%`}"></div>
            </div>
          </div>
          <span class="stat-mono w-4 flex-none text-right">{{ author.books }}</span>
        </li>
      </ol>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, type PropType} from 'vue';
import {useRouter} from 'vue-router';
import type {AuthorStat} from '@/composables/useStatsBreakdown';
import {formatPeriod} from '@/utils/period';
import {goToAuthor, isLinkableAuthor} from '@/utils/authorRoute';

export default defineComponent({
  props: {
    mode: {type: String, required: true},
    year: {type: Number, required: true},
    month: {type: Number, required: true},
    authors: {type: Array as PropType<AuthorStat[]>, default: () => []},
    loading: {type: Boolean, default: false},
  },
  setup(props) {
    const router = useRouter();
    const max = computed(() => Math.max(1, ...props.authors.map((a) => a.books)));
    const barWidth = (books: number) => Math.round((books / max.value) * 100);

    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));

    const viewAuthor = (author: string) => goToAuthor(router, author);

    return {barWidth, periodLabel, isKnown: isLinkableAuthor, viewAuthor};
  },
});
</script>

<style scoped>
.stat-mono {
  font-family: var(--font-mono), monospace;
  font-size: 12px;
  color: var(--color-ink-2);
}
</style>
