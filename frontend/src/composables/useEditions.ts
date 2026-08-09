import {ref} from 'vue';
import {useI18n} from 'vue-i18n';
import {fetchEditions} from '@/api/bookApi';
import type {BookSearchResult} from '@/types/book';

/**
 * Expands an Open Library work into the editions below it.
 *
 * Open Library keeps every translation and reprint as a separate edition of one
 * work, and the work itself carries no language — so "the German one" is a
 * choice only the user can make. Google volumes are already one per edition and
 * have nothing to expand.
 *
 * One row is open at a time. Fetched lists are kept, so comparing two works
 * costs one request each rather than one per click.
 */
export function useEditions() {
  const {t} = useI18n();
  const expanded = ref<string | null>(null);
  const editions = ref<BookSearchResult[]>([]);
  const loadingEditions = ref(false);
  const cache: Record<string, BookSearchResult[]> = {};

  const isWork = (book: BookSearchResult) =>
      book.source === 'openlibrary' && book.source_id.includes('/works/');

  const collapse = () => {
    expanded.value = null;
  };

  const toggleEditions = async (book: BookSearchResult) => {
    if (expanded.value === book.id) {
      collapse();
      return;
    }
    expanded.value = book.id;
    const cached = cache[book.id];
    if (cached) {
      editions.value = cached;
      return;
    }
    editions.value = [];
    loadingEditions.value = true;
    const result = await fetchEditions(book.source_id);
    // A second click while this one was in flight moved on — drop the answer.
    if (expanded.value === book.id) {
      cache[book.id] = result;
      editions.value = result;
    }
    loadingEditions.value = false;
  };

  const editionMeta = (edition: BookSearchResult) =>
      [
        edition.published_date,
        edition.publisher,
        edition.page_count ? t('search.pagesAbbr', {count: edition.page_count}) : null,
        edition.isbn13 || edition.isbn10,
      ].filter(Boolean).join(' · ');

  return {expanded, editions, loadingEditions, isWork, toggleEditions, collapse, editionMeta};
}
