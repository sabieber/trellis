import {ref, watch, type Ref} from 'vue';

// The four ways a book list renders. The order is the order of the selector.
export const LAYOUT_MODES = ['list', 'grid', 'shelf', 'pile'] as const;

export type LayoutMode = (typeof LAYOUT_MODES)[number];

export const LAYOUT_OPTIONS = LAYOUT_MODES.map((value) => ({value}));

/**
 * A layout mode that survives a reload.
 *
 * The shelf, author and browse views share the default key, so the preference
 * follows the user between them. A view over a different set of books (the goal
 * detail) passes its own key.
 */
export function useLayoutMode(storageKey = 'shelf-layout-mode'): Ref<LayoutMode> {
  const saved = localStorage.getItem(storageKey) as LayoutMode | null;
  const mode = ref<LayoutMode>(saved && LAYOUT_MODES.includes(saved) ? saved : 'list');
  watch(mode, (value) => localStorage.setItem(storageKey, value));
  return mode;
}
