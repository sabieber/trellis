import {ref} from 'vue';

// True from Tailwind's `md` breakpoint up. One listener for the whole app, for
// the few places where the size must be a number (`BookCover` picks its layout
// from the width) and a `md:` class cannot do the job.
const query = window.matchMedia('(min-width: 768px)');
const isWide = ref(query.matches);
query.addEventListener('change', (event) => (isWide.value = event.matches));

export function useIsWide() {
  return isWide;
}
