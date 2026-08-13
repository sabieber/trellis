<!-- Month calendar of the reading log: every day shows the cover of the book read
     that day, tapping a day opens the full list with the pages logged per book.
     Rolled by hand on the existing date helpers — daisyUI only themes third-party
     date pickers, none of which allow arbitrary content inside a day cell. -->
<template>
  <PageContainer :title="$t('calendar.title')" :description="$t('calendar.description')">
    <!-- Covers are sized so the whole month always fits the viewport, and the
         grid spreads across the page so the days get room to breathe rather than
         sitting shoulder to shoulder in a hugged-in card. -->
    <div class="bg-surface border border-line rounded-md p-3 sm:p-5 mx-auto w-full max-w-5xl"
         :style="{ '--cover': coverWidth + 'px' }">
      <header class="flex items-center gap-2 mb-3">
        <button
            class="flex items-center justify-center size-8 rounded-full text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
            :aria-label="$t('stats.prevPeriod')"
            @click="step(-1)"
        >
          <ChevronLeftIcon class="size-4.5"/>
        </button>
        <span class="t-title text-base flex-1 text-center select-none">{{ monthLabel }}</span>
        <button
            class="flex items-center justify-center size-8 rounded-full text-muted cursor-pointer hover:text-ink hover:bg-surface-2 transition-colors duration-150"
            :aria-label="$t('stats.nextPeriod')"
            @click="step(1)"
        >
          <ChevronRightIcon class="size-4.5"/>
        </button>
      </header>

      <div ref="gridEl">
        <div class="cal-grid mb-2">
          <span v-for="(label, index) in weekdayLabels" :key="index" class="cal-axis">{{ label }}</span>
        </div>

        <div v-if="loading" class="flex justify-center py-16">
          <span class="loading loading-spinner loading-sm"></span>
        </div>

        <div v-else class="cal-grid">
          <div v-for="cell in cells" :key="cell.id">
            <template v-if="cell.day">
              <div class="cal-num" :class="{ 'cal-num-today': cell.id === today }">{{ cell.day }}</div>
              <!-- Up to three covers, fanned back-to-front, so a busy day reads as
                   a stack at a glance; the badge carries the exact count. -->
              <button
                  v-if="cell.books.length"
                  class="cal-stack"
                  :aria-label="$t('calendar.dayAria', { date: cell.label, n: cell.books.length })"
                  @click="selected = cell"
              >
                <span
                    v-for="layer in fan(cell.books)"
                    :key="layer.book.reading_id"
                    class="cal-layer"
                    :style="layer.style"
                >
                  <BookCover
                      :title="layer.book.title || $t('common.untitled')"
                      :author="layer.book.author || ''"
                      :cover-url="resolvedCoverUrl(layer.book.book_id, layer.book.cover_url ?? undefined)"
                      :book-id="layer.book.book_id"
                      :width="coverTier"
                      @resolve-cover="onResolveCover"
                  />
                </span>
                <span class="cal-badges">
                  <!-- Flag: a book was finished on this day. -->
                  <span v-if="cell.finished" class="cal-flag" :title="$t('calendar.finished')">
                    <FlagIcon fill="color-mix(in srgb, currentColor 50%, transparent)" aria-hidden="true"/>
                  </span>
                  <span v-if="cell.books.length > 1" class="cal-badge">{{ cell.books.length }}</span>
                </span>
              </button>
              <div v-else class="cal-empty"></div>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- Day detail -->
    <div v-if="selected" class="modal modal-open">
      <div class="modal-box flex flex-col gap-4">
        <div>
          <h3 class="t-title text-lg">{{ selected.label }}</h3>
          <!-- Imported readings carry a finish date but no progress entries, so a
               day can hold books without a single logged page. -->
          <p class="t-meta mt-0.5">
            {{
              selected.pages
                  ? $t('calendar.dayTotal', {pages: selected.pages.toLocaleString(), n: selected.books.length})
                  : $t('calendar.dayBooks', selected.books.length)
            }}
          </p>
        </div>
        <ul class="flex flex-col gap-1 max-h-96 overflow-y-auto">
          <li v-for="book in selected.books" :key="book.reading_id">
            <RouterLink
                :to="{ name: 'book-detail', params: { id: book.book_id } }"
                class="flex items-center gap-3 px-2 py-2 rounded-md hover:bg-surface-2 transition-colors duration-150"
                @click="selected = null"
            >
              <BookCover
                  :title="book.title || $t('common.untitled')"
                  :author="book.author || ''"
                  :cover-url="resolvedCoverUrl(book.book_id, book.cover_url ?? undefined)"
                  :book-id="book.book_id"
                  :rating="book.rating"
                  :width="44"
                  @resolve-cover="onResolveCover"
              />
              <div class="flex-1 min-w-0">
                <div class="t-title text-sm truncate">{{ book.title || $t('common.untitled') }}</div>
                <div class="t-meta truncate">{{ book.author || $t('common.unknownAuthor') }}</div>
                <div class="mt-0.5 flex items-center gap-2">
                  <span v-if="book.pages" class="cal-mono">
                    {{ book.pages.toLocaleString() }} {{ $t('stats.unitPages', book.pages) }}
                  </span>
                  <span v-if="book.finished" class="badge badge-sm badge-primary">{{ $t('calendar.finished') }}</span>
                </div>
              </div>
            </RouterLink>
          </li>
        </ul>
        <div class="modal-action mt-0">
          <Button variant="ghost" block @click="selected = null">{{ $t('common.close') }}</Button>
        </div>
      </div>
      <div class="modal-backdrop" @click="selected = null"></div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {computed, defineComponent, onBeforeUnmount, onMounted, ref, watch} from 'vue';
import {useI18n} from 'vue-i18n';
import moment from 'moment';
import {ChevronLeftIcon, ChevronRightIcon} from '@lucide/vue';
import {FlagIcon} from '@lucide/vue';
import PageContainer from '@/components/PageContainer.vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import {useBookCovers} from '@/composables/useBookCovers';
import {useContainerWidth} from '@/composables/useContainerWidth';
import {dateKey, daysInMonth, mondayIndex} from '@/utils/activityHeat';
import {apiFetch} from '@/api/client';

/** A book read on a given day, as returned by `/api/stats/calendar`. */
interface CalendarBook {
  book_id: string;
  reading_id: string;
  title: string | null;
  author: string | null;
  cover_url: string | null;
  rating: number | null;
  pages: number;
  finished: boolean;
}

interface CalendarDay {
  date: string;
  pages: number;
  books: CalendarBook[];
}

/** One grid cell. Leading pad cells have no `day` and render as a gap. */
interface Cell extends CalendarDay {
  id: string;
  day: number;
  label: string;
  /** Whether any reading was finished on this day. */
  finished: boolean;
}

/** Desktop grid gap, mirrored from `.cal-grid` below. */
const GAP = 12;
/** Height a row costs on top of the cover: day number, its margin and the gap. */
const ROW_CHROME = 22;
/** Card and page padding below the grid, plus a little breathing room. */
const BOTTOM_GUTTER = 72;
const COVER_MIN = 34;
/** Only reached on a tall screen; the column width caps the cover before this. */
const COVER_MAX = 130;
/** Rotation of the fanned-out covers, front layer first. */
const FAN_ANGLES = [0, -8, 7];

export default defineComponent({
  components: {PageContainer, BookCover, Button, ChevronLeftIcon, ChevronRightIcon, FlagIcon},
  setup() {
    const {locale} = useI18n();
    const now = new Date();
    const year = ref(now.getFullYear());
    const month = ref(now.getMonth() + 1);
    const days = ref<CalendarDay[]>([]);
    const loading = ref(true);
    const selected = ref<Cell | null>(null);
    const today = dateKey(now);

    const gridEl = ref<HTMLElement | null>(null);
    const {containerWidth, setupObserver} = useContainerWidth(gridEl);
    /** Distance from the top of the grid to the bottom of the viewport. */
    const availableHeight = ref(0);

    const measureHeight = () => {
      const top = gridEl.value?.getBoundingClientRect().top ?? 0;
      availableHeight.value = window.innerHeight - top - BOTTOM_GUTTER;
    };

    onMounted(() => {
      setupObserver();
      measureHeight();
      window.addEventListener('resize', measureHeight);
    });
    onBeforeUnmount(() => window.removeEventListener('resize', measureHeight));

    /** Week rows the month spans, 4 to 6 depending on where the 1st falls. */
    const rowCount = computed(() =>
        Math.ceil((mondayIndex(new Date(year.value, month.value - 1, 1)) + daysInMonth(year.value, month.value)) / 7));

    // Sized off the viewport height, so the whole month always fits without
    // scrolling. Deliberately NOT off a measured width: the covers drive the
    // page height, so a width measurement would feed back through the scrollbar
    // appearing and disappearing — an endless resize loop. The column caps the
    // cover in CSS instead, where it cannot feed back into this computation.
    const coverWidth = computed(() => {
      const byHeight = (availableHeight.value / rowCount.value - ROW_CHROME) / 1.5;
      return Math.min(COVER_MAX, Math.max(COVER_MIN, Math.floor(byHeight)));
    });

    // What a cover actually ends up being once CSS caps it to the column. Only
    // BookCover's typographic tier (initial / title / author + title) rides on
    // this — it never sets a box size, so unlike a width-driven cover size it
    // cannot feed back into the layout. Without it the tier is picked for the
    // uncapped size and the title overflows its cover on a phone.
    const coverTier = computed(() =>
        Math.min(coverWidth.value, Math.floor(((containerWidth.value || 700) - GAP * 6) / 7)));

    /** Covers of a day, back-to-front, fanned out around the topmost one. */
    const fan = (dayBooks: CalendarBook[]) =>
        dayBooks
            .slice(0, FAN_ANGLES.length)
            .map((book, index) => ({book, style: {transform: `rotate(${FAN_ANGLES[index]}deg)`, zIndex: String(FAN_ANGLES.length - index)}}))
            .reverse();

    const weekdayLabels = computed(() =>
        Array.from({length: 7}, (_, i) => (locale.value, moment().isoWeekday(i + 1).format('ddd'))));

    const monthLabel = computed(() => (locale.value, moment({y: year.value, M: month.value - 1}).format('MMMM YYYY')));

    const load = async () => {
      loading.value = true;
      try {
        const res = await apiFetch('/api/stats/calendar', {
          method: 'POST',
          body: JSON.stringify({year: year.value, month: month.value}),
        });
        days.value = res.ok ? (await res.json()).days : [];
      } catch (e) {
        console.error('Failed to fetch calendar:', e);
        days.value = [];
      } finally {
        loading.value = false;
      }
    };

    watch([year, month], load, {immediate: true});

    const step = (delta: number) => {
      const next = new Date(year.value, month.value - 1 + delta, 1);
      year.value = next.getFullYear();
      month.value = next.getMonth() + 1;
    };

    const cells = computed<Cell[]>(() => {
      const byDate = new Map(days.value.map((day) => [day.date, day]));
      const first = new Date(year.value, month.value - 1, 1);
      const result: Cell[] = [];

      for (let index = 0; index < mondayIndex(first); index++) {
        result.push({id: `pad-${index}`, day: 0, label: '', date: '', pages: 0, books: [], finished: false});
      }
      for (let day = 1; day <= daysInMonth(year.value, month.value); day++) {
        const date = new Date(year.value, month.value - 1, day);
        const key = dateKey(date);
        const activity = byDate.get(key);
        result.push({
          id: key,
          day,
          // `LL` rather than a literal pattern, so the day/month order and the
          // ordinal dot follow the locale ("6. Januar 2025" / "January 6, 2025").
          label: moment(date).format('ddd, LL'),
          date: key,
          pages: activity?.pages ?? 0,
          books: activity?.books ?? [],
          finished: activity?.books.some((book) => book.finished) ?? false,
        });
      }
      return result;
    });

    const {resolvedCoverUrl, onResolveCover} = useBookCovers();

    return {
      gridEl, coverWidth, coverTier, weekdayLabels, monthLabel, cells, loading, selected, today, step, fan,
      resolvedCoverUrl, onResolveCover,
    };
  },
});
</script>

<style scoped>
.cal-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 8px; /* every pixel of gap is a pixel off the cover on a phone */
}

@media (min-width: 640px) {
  .cal-grid {
    gap: 12px;
  }
}

/* `--cover` is the height-derived size that keeps the month above the fold;
   `100%` caps it to the column on a screen too narrow for it. Columns are wider
   than the cover on a desktop, and that surplus is deliberate — it is the space
   between the days. Keeping the cap in CSS is also what keeps the size out of
   the resize loop a width-measuring version would cause: bigger covers make the
   page taller, the scrollbar appears, the measured width drops, and round it
   goes.

   Cells stay full-column width (no `justify-items: center`, which would make
   `100%` resolve against a shrink-to-fit box), so the covers centre themselves. */
.cal-stack,
.cal-empty {
  width: min(var(--cover), 100%);
  aspect-ratio: 2/3;
  margin-inline: auto;
}

.cal-stack {
  position: relative;
  display: block;
  cursor: pointer;
}

/* BookCover sizes itself in pixels; here the column decides instead, and the
   px prop only selects its typographic tier. */
.cal-stack :deep(.cover) {
  width: 100% !important;
}

.cal-layer {
  position: absolute;
  inset: 0;
  transform-origin: 50% 90%;
  transition: transform 150ms ease;
}

/* The whole stack tightens up on hover, the same lift the cards elsewhere use. */
.cal-stack:hover .cal-layer {
  transform: rotate(0deg) translateY(-2px);
}

.cal-axis {
  font-family: var(--font-mono), monospace;
  font-size: 10px;
  color: var(--color-muted);
  text-align: center;
}

.cal-num {
  font-family: var(--font-mono), monospace;
  font-size: 10px;
  line-height: 1;
  color: var(--color-muted);
  text-align: center;
  margin-bottom: 4px;
}

.cal-num-today {
  color: var(--color-green);
  font-weight: 500;
}

/* Days without a logged book keep the cover footprint, so the grid stays even. */
.cal-empty {
  border: 1px solid var(--color-line);
  border-radius: 5px;
}

/* Flag and count share the bottom-right corner rather than stacking, since a
   busy day can easily be both finished and multi-book. Both are sized off
   `--badge`, which shrinks on a phone: at full size the pair spans almost the
   entire 41px cover there. */
.cal-badges {
  --badge: 13px;
  position: absolute;
  z-index: 4; /* above every fanned layer */
  right: -3px;
  bottom: -3px;
  display: flex;
  align-items: center;
  gap: 2px;
}

.cal-badge {
  min-width: var(--badge);
  height: var(--badge);
  padding: 0 4px;
  border-radius: 999px;
  background: var(--color-green);
  color: var(--color-on-green);
  font-family: var(--font-mono), monospace;
  font-size: calc(var(--badge) * 0.64);
  line-height: var(--badge);
  text-align: center;
}

.cal-flag {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--badge);
  height: var(--badge);
  border-radius: 999px;
  background: var(--color-ink);
  color: var(--color-bg);
}

.cal-flag svg {
  width: calc(var(--badge) * 0.7);
  height: calc(var(--badge) * 0.7);
}

@media (min-width: 640px) {
  .cal-badges {
    --badge: 16px;
    gap: 3px;
  }
}

.cal-mono {
  font-family: var(--font-mono), monospace;
  font-size: 12px;
  color: var(--color-ink-2);
}
</style>
