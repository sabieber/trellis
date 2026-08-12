<template>
  <PageContainer :title="authorName" wide ref="pageContainer">
    <template #title>
      <h2 class="t-display text-2xl truncate">{{ authorName }}</h2>
      <p v-if="!loading && books.length" class="t-meta mt-1">
        <span>{{ $t('author.bookCount', { n: books.length }) }}</span>
        <span v-if="avgRating !== null"> · {{ $t('author.avgRating', { r: avgRating }) }}</span>
        <span v-if="totalPages > 0"> · {{ $t('author.totalPages', { n: totalPages.toLocaleString() }) }}</span>
      </p>
    </template>

    <template #title-button>
      <div v-if="!loading && books.length" class="flex items-center gap-2">
        <select v-model="sortBy" class="select select-sm w-36">
          <option value="added_at">{{ $t('shelf.sortAdded') }}</option>
          <option value="title">{{ $t('shelf.sortTitle') }}</option>
          <option value="author">{{ $t('shelf.sortAuthor') }}</option>
        </select>
        <LayoutModeSelect v-model="layoutMode"/>
      </div>
    </template>

    <!-- Open Library's author record. It loads on its own and never blocks the
         books below, so the screen stays useful for an author no catalog knows. -->
    <section v-if="info" class="flex gap-4 mb-8 pb-6 border-b border-line-soft">
      <img
          v-if="info.photo_url"
          :src="info.photo_url"
          :alt="info.name"
          class="w-24 rounded-sm shrink-0 self-start"
          loading="lazy"
          @error="info.photo_url = null"
      />
      <div class="min-w-0">
        <p v-if="authorMeta" class="t-meta">{{ authorMeta }}</p>
        <p v-if="info.alternate_names.length" class="t-meta mt-0.5">
          {{ $t('author.alsoKnownAs', { names: info.alternate_names.join(', ') }) }}
        </p>
        <div
            v-if="info.bio"
            class="text-ink-dim text-sm leading-relaxed mt-2 [&_p]:mb-3 [&_p:last-child]:mb-0 [&_a]:text-green-soft [&_a]:underline [&_a]:underline-offset-2 hover:[&_a]:text-green"
            :class="{ 'line-clamp-4': !bioExpanded }"
            v-html="info.bio"
        ></div>
        <button
            v-if="info.bio && info.bio.length > 400"
            class="t-meta underline underline-offset-2 mt-2"
            @click="bioExpanded = !bioExpanded"
        >
          {{ bioExpanded ? $t('author.showLess') : $t('author.showMore') }}
        </button>
        <div class="flex flex-wrap gap-2 mt-4">
          <Button v-for="link in authorLinks" :key="link.url" variant="ghost" :href="link.url">
            <ExternalLinkIcon class="size-4"/>
            {{ link.title }}
          </Button>
        </div>
      </div>
    </section>

    <div v-if="loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <template v-else-if="books.length">
      <!-- Both sections are lists of books, so each says which list it is. -->
      <h3 class="t-eyebrow mb-4">{{ $t('author.onYourShelf') }}</h3>
      <BookLayout :books="sortedBooks" :mode="layoutMode"/>
    </template>

    <div v-else class="t-meta text-center py-12">{{ $t('shelf.noBooks') }}</div>

    <!-- Works the user does not own. They are catalog hits, so they get the
         search result row and link to the external-lookup detail view. The
         layout mode above governs the shelf only: it sizes spines and piles
         from what the user owns, which a catalog hit has no data for. -->
    <section v-if="info?.works.length" class="mt-10 pt-6 border-t border-line-soft">
      <h3 class="t-eyebrow mb-4">{{ $t('author.moreWorks') }}</h3>
      <BookResultRow v-for="work in info.works" :key="work.id" :book="work"/>
    </section>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, computed, onMounted, watch} from 'vue';
import {useRoute} from 'vue-router';
import {useI18n} from 'vue-i18n';
import {ExternalLinkIcon} from '@lucide/vue';
import PageContainer from '@/components/PageContainer.vue';
import BookLayout from '@/components/shelf/BookLayout.vue';
import LayoutModeSelect from '@/components/shelf/LayoutModeSelect.vue';
import BookResultRow from '@/components/ui/BookResultRow.vue';
import Button from '@/components/ui/Button.vue';
import {apiFetch} from '@/api/client';
import {fetchAuthorInfo, type AuthorInfo} from '@/api/bookApi';
import {useLayoutMode} from '@/composables/useLayoutMode';
import type {ShelfBook} from '@/types/shelf';

export default defineComponent({
  components: {
    PageContainer, BookLayout, LayoutModeSelect, BookResultRow, Button, ExternalLinkIcon,
  },
  setup() {
    const route = useRoute();
    const {t} = useI18n();
    const books = ref<ShelfBook[]>([]);
    const loading = ref(true);
    const info = ref<AuthorInfo | null>(null);
    const bioExpanded = ref(false);
    const sortBy = ref<'added_at' | 'title' | 'author'>('added_at');
    const pageContainer = ref<any>(null);
    const layoutMode = useLayoutMode();

    const authorName = computed(() => route.params.name as string);

    const avgRating = computed(() => {
      const rated = books.value.filter((b) => b.rating != null);
      if (!rated.length) return null;
      const sum = rated.reduce((acc, b) => acc + (b.rating as number), 0);
      return (sum / rated.length).toFixed(1);
    });

    const authorMeta = computed(() => {
      if (!info.value) return '';
      const parts: string[] = [];
      if (info.value.birth_date) parts.push(t('author.born', {date: info.value.birth_date}));
      if (info.value.death_date) parts.push(t('author.died', {date: info.value.death_date}));
      // How much of the catalog's author the shelf holds. Open Library counts
      // every edition record as a work, so the total runs high.
      if (info.value.work_count) {
        parts.push(t('author.coverage', {owned: books.value.length, total: info.value.work_count}));
      }
      return parts.join(' · ');
    });

    // The Open Library page itself leads; the record's own links follow.
    const authorLinks = computed(() => {
      if (!info.value) return [];
      return [
        {title: 'Open Library', url: `https://openlibrary.org/authors/${info.value.key}`},
        ...info.value.links,
      ];
    });

    const totalPages = computed(() =>
        books.value.reduce((acc, b) => acc + (b.page_count || 0), 0)
    );

    const sortedBooks = computed(() => {
      const arr = [...books.value];
      if (sortBy.value === 'title') {
        arr.sort((a, b) => a.title.localeCompare(b.title));
      } else if (sortBy.value === 'author') {
        arr.sort((a, b) => a.author.localeCompare(b.author));
      } else {
        arr.sort((a, b) => b.added_at.localeCompare(a.added_at));
      }
      return arr;
    });

    const fetchAuthorBooks = async (author: string) => {
      loading.value = true;
      try {
        const response = await apiFetch('/api/authors/books', {
          method: 'POST',
          body: JSON.stringify({author}),
        });
        if (response.ok) {
          const data = await response.json();
          books.value = data.books;
        } else {
          console.error('Failed to fetch author books:', await response.json());
        }
      } catch (error) {
        console.error('Failed to fetch author books:', error);
      } finally {
        loading.value = false;
      }
    };

    const loadAuthorInfo = async (name: string) => {
      info.value = null;
      bioExpanded.value = false;
      info.value = await fetchAuthorInfo(name);
    };

    const load = (name: string) => {
      fetchAuthorBooks(name);
      loadAuthorInfo(name);
    };

    onMounted(() => load(authorName.value));
    // Re-fetch when navigating between authors without unmounting the view.
    watch(authorName, (name) => load(name));

    return {
      books, sortedBooks, loading, sortBy, authorName,
      avgRating, totalPages, layoutMode, pageContainer,
      info, bioExpanded, authorMeta, authorLinks,
    };
  },
});
</script>
