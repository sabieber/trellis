import {computed, ref} from 'vue';
import {useI18n} from 'vue-i18n';
import {fetchEditions} from '@/api/bookApi';
import {editionLanguages, languageLabel, normalizeLanguage} from '@/utils/editionLanguages';
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
  const {t, locale} = useI18n();
  const expanded = ref<string | null>(null);
  const editions = ref<BookSearchResult[]>([]);
  const loadingEditions = ref(false);
  const cache: Record<string, BookSearchResult[]> = {};

  const isWork = (book: BookSearchResult) =>
      book.source === 'openlibrary' && book.source_id.includes('/works/');

  // Every language group starts closed, so a work answers "which languages do
  // I have?" in one screen. Opening one is the second question.
  const openGroups = ref<string[]>([]);

  const isGroupOpen = (language: string) => openGroups.value.includes(language);

  const toggleGroup = (language: string) => {
    openGroups.value = isGroupOpen(language)
        ? openGroups.value.filter((open) => open !== language)
        : [...openGroups.value, language];
  };

  const collapse = () => {
    expanded.value = null;
    openGroups.value = [];
  };

  const toggleEditions = async (book: BookSearchResult) => {
    if (expanded.value === book.id) {
      collapse();
      return;
    }
    expanded.value = book.id;
    openGroups.value = [];
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

  /**
   * The editions to show, in groups of one language.
   *
   * The language preference filters the list. An edition that states no
   * language stays — Open Library leaves that field empty on a third of its
   * records, and dropping them hides untagged English hardbacks as well.
   *
   * The preferred languages come first, in the order the user picked them, then
   * the rest, then the editions of unknown language.
   */
  const editionGroups = computed(() => {
    const wanted = editionLanguages.value;
    const groups = new Map<string, BookSearchResult[]>();

    for (const edition of editions.value) {
      const language = normalizeLanguage(edition.language);
      if (wanted.length && language && !wanted.includes(language)) continue;
      const key = language ?? '';
      const group = groups.get(key);
      if (group) group.push(edition);
      else groups.set(key, [edition]);
    }

    const rank = (language: string) => {
      if (!language) return Number.MAX_SAFE_INTEGER;
      const preferred = wanted.indexOf(language);
      return preferred >= 0 ? preferred : wanted.length;
    };

    return [...groups.entries()]
        .sort(([a], [b]) => rank(a) - rank(b) || a.localeCompare(b))
        .map(([language, group]) => ({
          language,
          label: language ? languageLabel(language, locale.value) : t('search.unknownLanguage'),
          editions: group,
        }));
  });

  // Nothing left after the filter is a different answer than nothing at all.
  const hiddenByLanguage = computed(() => !editionGroups.value.length && editions.value.length > 0);

  const editionMeta = (edition: BookSearchResult) =>
      [
        edition.published_date,
        edition.publisher,
        edition.page_count ? t('search.pagesAbbr', {count: edition.page_count}) : null,
        edition.isbn13 || edition.isbn10,
      ].filter(Boolean).join(' · ');

  return {
    expanded,
    editions,
    editionGroups,
    hiddenByLanguage,
    isGroupOpen,
    toggleGroup,
    loadingEditions,
    isWork,
    toggleEditions,
    collapse,
    editionMeta,
  };
}
