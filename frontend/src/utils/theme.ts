import {ref, watch} from 'vue';

/**
 * The colour theme of this device.
 *
 * Unlike the other preferences (`utils/userSettings.ts`) the theme does not live
 * on the user row. It answers to the screen, not to the reader: the eInk reader
 * wants paper, the phone in bed wants the dark garden. One account value would
 * flip the phone every time the eReader changed it.
 *
 * `index.html` sets the attribute before the first paint from the same storage
 * key, so a light device never flashes dark. Keep both in sync.
 */
export const THEMES = ['dark', 'light', 'eink', 'system'] as const;

export type Theme = (typeof THEMES)[number];

const STORAGE_KEY = 'theme';

/** The daisyUI theme name behind each choice. `system` resolves to dark or light. */
const DATA_THEME: Record<Exclude<Theme, 'system'>, string> = {
  dark: 'trellis',
  light: 'trellis-light',
  eink: 'trellis-eink',
};

function stored(): Theme {
  try {
    const value = localStorage.getItem(STORAGE_KEY) as Theme | null;
    return value && THEMES.includes(value) ? value : 'dark';
  } catch {
    return 'dark';
  }
}

export const theme = ref<Theme>(stored());

const darkQuery = window.matchMedia('(prefers-color-scheme: dark)');

function apply(): void {
  const resolved = theme.value === 'system' ? (darkQuery.matches ? 'dark' : 'light') : theme.value;
  document.documentElement.dataset.theme = DATA_THEME[resolved];
}

watch(theme, (value) => {
  try {
    localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // The choice still holds for this visit.
  }
  apply();
});

darkQuery.addEventListener('change', () => {
  if (theme.value === 'system') apply();
});

apply();
