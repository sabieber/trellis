import {ref} from 'vue';

/**
 * Which edition languages the reader wants to see.
 *
 * Open Library tags an edition with a MARC language code, and a work like "The
 * Two Towers" has editions in twenty of them. An empty list means "show all",
 * which is where everyone starts.
 *
 * The preference lives on the user row and follows the reader between devices.
 * localStorage only caches it, so the first render after a reload has an answer
 * before `utils/userSettings.ts` gets one from the server. Write through
 * `changeEditionLanguages` — the setter here does not reach the server.
 */
const STORAGE_KEY = 'editionLanguages';

/**
 * Reduces any language code to its BCP-47 primary tag: Open Library writes
 * MARC codes and mixes the two variants some languages have (`ger` and `deu`,
 * `fre` and `fra`). `Intl.Locale` maps all of them onto one tag, so `ger`,
 * `deu` and `de` become the same group and match the same preference.
 */
export function normalizeLanguage(code?: string | null): string | null {
  if (!code) return null;
  try {
    return new Intl.Locale(code).language;
  } catch {
    return code.toLowerCase();
  }
}

/**
 * The language's name in the reader's own language, from the browser's CLDR
 * data. Codes CLDR does not know come back unchanged — show those as-is.
 */
export function languageLabel(code: string, locale: string): string {
  try {
    return new Intl.DisplayNames([locale], {type: 'language'}).of(code) ?? code.toUpperCase();
  } catch {
    return code.toUpperCase();
  }
}

/**
 * Every language the browser can name, sorted by name in the reader's own
 * language.
 *
 * `Intl` names a code but does not enumerate, and `Intl.supportedValuesOf` has
 * no "language" key — so we hand it the whole two-letter ISO 639-1 space and
 * keep what comes back named. An unknown code is echoed back unchanged, which
 * is the test. That is 676 lookups and ~2 ms, once per locale, and it stays
 * exactly as complete as the browser running it: no code list to maintain, and
 * no options reading "grc" for a language this browser cannot name.
 *
 * Only canonical codes make the list. The deprecated aliases (`iw` for Hebrew,
 * `mo` for Romanian, `tw` for Akan, …) carry the same name as the code they
 * were replaced by and would otherwise show up as a duplicate option.
 */
const cache: Record<string, { code: string; label: string }[]> = {};

export function editionLanguageOptions(locale: string) {
  const cached = cache[locale];
  if (cached) return cached;

  const options = [];
  for (let first = 97; first <= 122; first++) {
    for (let second = 97; second <= 122; second++) {
      const code = String.fromCharCode(first, second);
      if (normalizeLanguage(code) !== code) continue;
      const label = languageLabel(code, locale);
      if (label.toLowerCase() !== code) options.push({code, label});
    }
  }
  options.sort((a, b) => a.label.localeCompare(b.label, locale));
  cache[locale] = options;
  return options;
}

function stored(): string[] {
  try {
    const raw = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]');
    // Normalized on the way in as well: a stored MARC code still matches.
    return Array.isArray(raw) ? raw.map(normalizeLanguage).filter((code): code is string => !!code) : [];
  } catch {
    return [];
  }
}

export const editionLanguages = ref<string[]>(stored());

export function setEditionLanguages(codes: string[]) {
  editionLanguages.value = codes;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(codes));
}
