<!-- A compact ranked list of the period's standout books, used both for the
     best-rated books (trailing star rating) and the most-read books (trailing
     count of finished readings). -->
<template>
  <div class="lg:h-full lg:flex lg:flex-col">
    <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 mb-3">
      <div class="flex items-baseline gap-2">
        <h2 class="t-eyebrow">{{ $t(title) }}</h2>
        <span class="t-meta">{{ periodLabel }}</span>
      </div>
    </div>

    <div class="bg-surface border border-line rounded-md p-4 flex-1 flex flex-col justify-center">
      <div v-if="loading" class="flex justify-center py-14">
        <span class="loading loading-spinner loading-sm"></span>
      </div>
      <div v-else-if="books.length === 0" class="t-meta text-center py-14">
        {{ $t(emptyText) }}
      </div>

      <ol v-else class="space-y-3">
        <li
            v-for="(book, index) in books"
            :key="index"
            class="relative flex items-center gap-3 -mx-2 px-2 py-1 rounded-md hover:bg-surface-2 transition-colors duration-150"
        >
          <BookCover
              :title="book.title"
              :author="book.author"
              :cover-url="resolvedCoverUrl(book.book_id, book.cover_url ?? undefined)"
              :book-id="book.book_id"
              :width="34"
              @resolve-cover="onResolveCover"
          />
          <div class="flex-1 min-w-0">
            <div class="t-title text-sm truncate">
              <RouterLink
                  class="stretched-link"
                  :to="{ name: 'book-detail', params: { id: book.book_id } }"
              >{{ book.title }}</RouterLink>
            </div>
            <div class="t-meta truncate">
              <RouterLink
                  v-if="isLinkableAuthor(book.author)"
                  class="relative z-1 hover:text-green-soft hover:underline transition-colors duration-150"
                  :to="authorRoute(book.author)"
              >{{ book.author }}</RouterLink>
              <span v-else>{{ book.author }}</span>
            </div>
          </div>
          <span v-if="metric === 'rating'" class="flex-none flex items-center gap-0.5 text-gold">
            <!-- Thumbs mode has no score to print, so the thumb stands alone. -->
            <component
                v-if="thumbs"
                :is="tendency(book.rating) === -1 ? ThumbsDownIcon : ThumbsUpIcon"
                class="size-3.5"
                :class="{ 'rotate-[-90deg]': tendency(book.rating) === 0 }"
            />
            <template v-else>
              <!-- Same half-fill as Rating.vue: a full fill with no stroke merges
                   the petals and centre into a blob at this size. -->
              <FlowerIcon class="size-3.5" fill="color-mix(in srgb, currentColor 50%, transparent)"/>
              <span class="stat-mono">{{ book.rating }}</span>
            </template>
          </span>
          <span v-else class="stat-mono flex-none">{{ $t('stats.timesRead', { n: book.readings }) }}</span>
        </li>
      </ol>
    </div>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, type PropType} from 'vue';
import {FlowerIcon, ThumbsDownIcon, ThumbsUpIcon} from '@lucide/vue';
import BookCover from '@/components/ui/BookCover.vue';
import type {BookStat} from '@/composables/useStatsBreakdown';
import {useBookCovers} from '@/composables/useBookCovers';
import {formatPeriod} from '@/utils/period';
import {authorRoute, isLinkableAuthor} from '@/utils/authorRoute';
import {ratingMode, tendency} from '@/utils/ratingMode';

export default defineComponent({
  components: {FlowerIcon, ThumbsDownIcon, ThumbsUpIcon, BookCover},
  props: {
    mode: {type: String, required: true},
    year: {type: Number, required: true},
    month: {type: Number, required: true},
    title: {type: String, required: true},
    emptyText: {type: String, required: true},
    metric: {type: String as PropType<'rating' | 'readings'>, required: true},
    books: {type: Array as PropType<BookStat[]>, default: () => []},
    loading: {type: Boolean, default: false},
  },
  setup(props) {
    const periodLabel = computed(() => formatPeriod(props.mode, props.year, props.month));
    const {resolvedCoverUrl, onResolveCover} = useBookCovers();
    const thumbs = computed(() => ratingMode.value === 'thumbs');

    return {
      periodLabel, resolvedCoverUrl, onResolveCover, authorRoute, isLinkableAuthor,
      thumbs, tendency, ThumbsDownIcon, ThumbsUpIcon,
    };
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
