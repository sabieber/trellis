<template>
  <div class="min-h-screen">
    <div class="flex flex-col">
      <div class="px-4 pt-4 pb-2">
        <Button variant="ghost" class="px-3.5! py-2! text-[13px]!" @click="$router.back()">
          <ChevronLeftIcon class="size-4"/>
          {{ $t('common.back') }}
        </Button>
      </div>

      <div v-if="loading" class="flex justify-center py-12">
        <span class="loading loading-spinner loading-lg"></span>
      </div>

      <div v-else-if="book" class="px-4 pb-8">
        <div class="flex gap-5 mb-6">
          <BookCover
              :title="book.title || $t('common.untitled')"
              :author="book.authors?.join(', ') || ''"
              :width="128"
              :cover-url="book.cover_url"
              class="flex-none"
          />
          <div class="flex flex-col justify-end min-w-0">
            <h1 class="t-display text-[22px]">{{ book.title }}</h1>
            <p v-if="book.subtitle" class="t-meta text-[15px] mt-1">{{ book.subtitle }}</p>
            <p class="t-meta text-sm mt-1.5">{{ book.authors?.join(', ') }}</p>
            <div v-if="book.average_rating" class="flex items-center gap-2 mt-2.5">
              <!-- The catalog average, not the user's own verdict: always stars. -->
              <Rating :rating="book.average_rating" stars/>
              <span class="t-meta">
                {{ $t('searchDetail.avgRating', { rating: book.average_rating.toFixed(1) }) }}
                <span v-if="book.ratings_count"> · {{ $t('searchDetail.ratingsCount', { count: book.ratings_count }) }}</span>
              </span>
            </div>
            <p class="t-meta mt-2">
              {{ book.published_year }}
              <span v-if="book.page_count"> · {{ $t('search.pagesAbbr', { count: book.page_count }) }}</span>
            </p>
            <p v-if="book.series" class="t-meta mt-1">
              <RouterLink
                  class="hover:text-green-soft hover:underline transition-colors duration-150"
                  :to="seriesRoute(book.series.key)"
              >{{ book.series.position
                    ? $t('bookDetail.seriesEntry', { name: book.series.name, position: book.series.position })
                    : book.series.name }}</RouterLink>
            </p>
          </div>
        </div>

        <div v-if="categories.length" class="flex flex-wrap gap-2 mb-6">
          <Chip v-for="c in categories" :key="c">{{ c }}</Chip>
        </div>

        <!-- Named grid areas resort the same elements between layouts: on mobile a
             single column (Add · Description · Details · Link); on desktop the
             description fills the left column while the rail (Add/Details/Link)
             sits on the right. The rail is a real flex column on desktop (so it
             stays packed at the top regardless of description length) and is
             dissolved via `display: contents` on mobile so its items can be
             interleaved with the description by the grid areas. -->
        <div class="detail-layout">
          <div v-if="book.description" class="da-desc min-w-0">
            <h2 class="t-eyebrow mb-2">{{ $t('common.description') }}</h2>
            <div
                class="text-ink-dim text-sm leading-relaxed [&_p]:mb-3 [&_p:last-child]:mb-0 [&_a]:text-green-soft [&_a]:underline [&_a]:underline-offset-2 hover:[&_a]:text-green"
                v-html="book.description"
            ></div>
          </div>

          <div class="rail">
            <Button class="da-add" block @click="showShelfModal = true">
              <PlusIcon class="size-4"/>
              {{ $t('searchDetail.addToLibrary') }}
            </Button>

            <div v-if="hasMeta" class="da-details">
              <h2 class="t-eyebrow mb-2">{{ $t('searchDetail.details') }}</h2>
              <dl class="flex flex-col">
                <div v-if="book.publisher" class="flex items-center justify-between gap-4 py-2.5 border-b border-line-soft">
                  <dt class="t-meta flex-none">{{ $t('searchDetail.publisher') }}</dt>
                  <dd class="text-sm text-ink text-right min-w-0 truncate">{{ book.publisher }}</dd>
                </div>
                <div v-if="book.published_date" class="flex items-center justify-between gap-4 py-2.5 border-b border-line-soft">
                  <dt class="t-meta flex-none">{{ $t('searchDetail.published') }}</dt>
                  <dd class="text-sm text-ink text-right">{{ book.published_date }}</dd>
                </div>
                <div v-if="book.language" class="flex items-center justify-between gap-4 py-2.5 border-b border-line-soft">
                  <dt class="t-meta flex-none">{{ $t('searchDetail.language') }}</dt>
                  <dd class="text-sm text-ink text-right uppercase">{{ book.language }}</dd>
                </div>
                <div v-if="isbn" class="flex items-center justify-between gap-4 py-2.5 border-b border-line-soft">
                  <dt class="t-meta flex-none">{{ $t('searchDetail.isbn') }}</dt>
                  <dd class="text-sm text-ink text-right">{{ isbn }}</dd>
                </div>
              </dl>
            </div>

            <Button v-if="book.info_link" class="da-ext" variant="ghost" block @click="openSource">
              <ExternalLinkIcon class="size-4"/>
              {{ $t('searchDetail.viewSource', { source: sourceLabel }) }}
            </Button>
          </div>
        </div>
      </div>

      <div v-else class="t-meta text-center py-8 px-4">{{ $t('common.bookNotFound') }}</div>
    </div>

    <AddToShelfPopup
        v-if="showShelfModal && book"
        :book="book"
        @close="showShelfModal = false"
        @toast="onShelfToast"
    />

    <div v-if="toastMessage" class="toast toast-top toast-center pt-4 z-50">
      <div :class="`alert ${toastType}`">
        <span>{{ toastMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {useI18n} from 'vue-i18n';
import {ChevronLeftIcon, PlusIcon, ExternalLinkIcon} from "@lucide/vue";
import {fetchBookDetail} from '@/api/bookApi';
import {seriesRoute} from '@/utils/seriesRoute';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import Chip from '@/components/ui/Chip.vue';
import Rating from '@/components/ui/Rating.vue';
import AddToShelfPopup from '@/components/AddToShelfPopup.vue';
import type {BookSearchResult} from '@/types/book';

export default defineComponent({
  components: {ChevronLeftIcon, PlusIcon, ExternalLinkIcon, BookCover, Button, Chip, Rating, AddToShelfPopup},
  setup() {
    const {t} = useI18n();
    const route = useRoute();
    const router = useRouter();
    const book = ref<BookSearchResult | null>(null);
    const loading = ref(true);
    const showShelfModal = ref(false);
    const toastMessage = ref('');
    const toastType = ref('');

    // Prefer the full detail category list; fall back to the single list-row
    // category. Capped so a book's long Open Library subject list stays readable.
    const categories = computed(() =>
        (book.value?.categories?.length ? book.value.categories : book.value?.category ? [book.value.category] : []).slice(0, 8));
    const isbn = computed(() => book.value?.isbn13 || book.value?.isbn10 || '');
    const hasMeta = computed(() =>
        !!(book.value?.publisher || book.value?.published_date || book.value?.language || isbn.value));
    const sourceLabel = computed(() => (book.value ? t(`search.source.${book.value.source}`) : ''));

    const openSource = () => {
      if (book.value?.info_link) window.open(book.value.info_link, '_blank', 'noopener');
    };


    const showToast = (message: string, type: string) => {
      toastMessage.value = message;
      toastType.value = type;
      setTimeout(() => {
        toastMessage.value = '';
        toastType.value = '';
      }, 3000);
    };

    const onShelfToast = (payload: { message: string; type: string }) => {
      showToast(payload.message, payload.type);
    };

    onMounted(async () => {
      const compositeId = route.params.id as string;
      const colonIndex = compositeId.indexOf(':');
      const source = compositeId.substring(0, colonIndex);
      const sourceId = compositeId.substring(colonIndex + 1);
      book.value = await fetchBookDetail(source, sourceId);
      loading.value = false;
    });

    return {
      book,
      loading,
      showShelfModal,
      toastMessage,
      toastType,
      categories,
      isbn,
      hasMeta,
      sourceLabel,
      openSource,
      seriesRoute,
      onShelfToast,
    };
  },
});
</script>

<style scoped>
/* Mobile: one column, ideal reading order. Named areas decide the order, so the
   rail's items can sit between the description regardless of DOM order. */
.detail-layout {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-areas:
    "add"
    "desc"
    "details"
    "ext";
  row-gap: 1.5rem;
}

/* On mobile the rail is dissolved so its children become grid items and get
   placed individually by the areas above. */
.detail-layout > .rail {
  display: contents;
}

.da-add {
  grid-area: add;
}

.da-desc {
  grid-area: desc;
}

.da-details {
  grid-area: details;
}

.da-ext {
  grid-area: ext;
}

/* Desktop: description left (2/3), rail right (1/3). The rail becomes a real flex
   column again — a single grid cell — so Add/Details/Link stay packed at the top
   even when the description is much taller (no row-distribution gaps). */
@media (min-width: 1024px) {
  .detail-layout {
    grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
    grid-template-areas: none;
    column-gap: 2.5rem;
    align-items: start;
    row-gap: 0;
  }

  .detail-layout > .rail {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    grid-column: 2;
    grid-row: 1;
  }

  .da-desc {
    grid-area: auto;
    grid-column: 1;
    grid-row: 1;
  }
}
</style>
