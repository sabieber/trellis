import {apiFetch} from '@/api/client';
import {useAuthStore} from '@/stores/auth';
import {setLocale, type Locale} from '@/i18n';
import {editionLanguages, setEditionLanguages} from '@/utils/editionLanguages';
import {ratingMode, type RatingMode} from '@/utils/ratingMode';

/**
 * Every preference the user picks, kept on the user row.
 *
 * A setting that differs between the phone and the desktop is a setting nobody
 * can rely on, so the server holds the truth and this device only caches it.
 * The caches stay because the app needs an answer before it has a user: the
 * login page renders in a language, and the first render must not wait for a
 * request. On a login the server value replaces the cached one.
 *
 * Every setter writes the local value first and the server afterwards. A save
 * that fails costs the setting on the next device, not the click.
 */
interface UserSettings {
  rating_mode?: string;
  locale?: string;
  edition_languages?: string[];
}

function save(patch: UserSettings): void {
  // The login page can change the language before there is a user to save it to.
  if (!useAuthStore().token) return;

  apiFetch('/api/user/settings', {method: 'PUT', body: JSON.stringify(patch)})
      .catch((error) => console.error('Failed to save the settings:', error));
}

export function changeRatingMode(mode: RatingMode): void {
  ratingMode.value = mode;
  save({rating_mode: mode});
}

export function changeLocale(locale: Locale): void {
  setLocale(locale);
  save({locale});
}

export function changeEditionLanguages(codes: string[]): void {
  setEditionLanguages(codes);
  save({edition_languages: codes});
}

/**
 * Reads the settings after a login and applies them.
 *
 * A setting the account does not carry yet is filled from this device, which is
 * how the preferences a reader picked before this all lived on the server find
 * their way up there — once, without anybody re-picking anything.
 */
export async function loadSettings(): Promise<void> {
  let settings: UserSettings;
  try {
    const response = await apiFetch('/api/user/settings');
    if (!response.ok) return;
    settings = await response.json();
  } catch {
    // Keep the cached values; a preference is not worth a blocking error.
    return;
  }

  if (settings.rating_mode === 'thumbs' || settings.rating_mode === 'stars') {
    ratingMode.value = settings.rating_mode;
  }

  // Only a language the reader picked moves up. The one the browser guessed is
  // not a choice, and writing it would freeze a guess onto the account.
  const picked = localStorage.getItem('locale');
  if (settings.locale === 'en' || settings.locale === 'de') {
    setLocale(settings.locale);
  } else if (picked === 'en' || picked === 'de') {
    save({locale: picked});
  }

  const codes = settings.edition_languages ?? [];
  if (codes.length) {
    setEditionLanguages(codes);
  } else if (editionLanguages.value.length) {
    save({edition_languages: editionLanguages.value});
  }
}
