<!-- "Surprise me": a breeze blows through a small stretch of the shelf, the
     spines sway, and one book is carried off the board — its cover rises out
     of the gap. Clicking the cover opens the book detail. -->
<template>
  <div class="modal modal-open">
    <div class="picker" @click.self="$emit('close')">
      <Transition name="fade" mode="out-in">
        <p class="t-eyebrow" :key="phase === 'gust' ? 'blowing' : 'picked'">
          {{ phase === 'gust' ? $t('shelf.breeze') : $t('shelf.randomPick') }}
        </p>
      </Transition>

      <div class="cover-slot">
        <div v-if="pick && phase === 'revealed'" class="reveal">
          <RouterLink :to="{ name: 'book-detail', params: { id: pick.id } }">
          <BookCover
              :title="pick.title"
              :author="pick.author"
              :width="150"
              :cover-url="resolvedCoverUrl(pick.id, bookCoverUrl(pick))"
              :book-id="pick.id"
              :rating="pick.rating"
              :has-note="pick.has_notes"
              hoverable
              @resolve-cover="onResolveCover"
          />
          </RouterLink>
          <div class="reveal-text">
            <p class="t-title">{{ pick.title }}</p>
            <p class="t-meta mt-0.5">{{ pick.author }}</p>
          </div>
        </div>
      </div>

      <div ref="boardRef" class="board">
        <ShelfBoardView
            ref="boardViewRef"
            :books="strip"
            :spine-height="132"
            :container-width="boardWidth"
            :gust="phase === 'gust'"
            :picked-id="phase === 'gust' ? null : pick?.id ?? null"
            :picked-taken="phase === 'revealed'"
        />
        <div v-if="phase !== 'revealed'" class="leaves">
          <svg
              v-for="(leaf, i) in LEAVES"
              :key="i"
              class="leaf"
              viewBox="0 -21 21 22"
              :style="{ top: leaf.y + '%', animationDelay: leaf.delay + 'ms', '--s': leaf.scale }"
          >
            <path :d="LEAF"/>
          </svg>
        </div>
      </div>

      <div class="flex gap-2">
        <Button variant="soft" :disabled="phase !== 'revealed'" @click="blow">{{ $t('shelf.blowAgain') }}</Button>
        <Button variant="ghost" @click="$emit('close')">{{ $t('common.close') }}</Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script setup lang="ts">
import {computed, nextTick, onMounted, onUnmounted, ref, type ComponentPublicInstance} from 'vue';
import BookCover from '@/components/ui/BookCover.vue';
import Button from '@/components/ui/Button.vue';
import ShelfBoardView from '@/components/shelf/ShelfBoardView.vue';
import {bookCoverUrl} from '@/utils/coverUrl';
import {spineWidth} from '@/utils/bookColorway';
import {useBookCovers} from '@/composables/useBookCovers';
import {useContainerWidth} from '@/composables/useContainerWidth';
import type {ShelfBook} from '@/types/shelf';

const props = defineProps<{ books: ShelfBook[] }>();
const emit = defineEmits<{ close: [] }>();

// The plump leaf of the signature vine (see VineProgress.vue), drawn in a
// ~20×21 box with its stalk at the origin.
const LEAF = 'M0 0 C 0 -12.6 7.2 -19.8 18 -20.7 C 20.7 -9.9 12.6 -0.9 0 0 Z';
// Delays stay under SETTLE_MS so every leaf has drifted off before the reveal
// unmounts the layer — otherwise they blink out mid-flight.
const LEAVES = [
  {y: 18, delay: 0, scale: 0.9},
  {y: 62, delay: 110, scale: 0.55},
  {y: 34, delay: 260, scale: 1.15},
  {y: 78, delay: 390, scale: 0.75},
  {y: 8, delay: 520, scale: 0.65},
  {y: 50, delay: 640, scale: 1},
];
// ponytail: one screenful of shelf, not all 400 books — the breeze needs a
// stretch of spines to blow through, not the whole library. More than enough
// candidates to fill any board width; the rest are dropped when packing.
const CANDIDATES = 24;
/** Must match the gap of .shelf-books in ShelfBoardView. */
const SPINE_GAP = 3;

const {resolvedCoverUrl, onResolveCover} = useBookCovers();
const boardRef = ref<HTMLElement | null>(null);
const boardViewRef = ref<ComponentPublicInstance | null>(null);
const {containerWidth, setupObserver} = useContainerWidth(boardRef);
// .shelf-books carries 8px of padding on each side.
const boardWidth = computed(() => Math.max(0, containerWidth.value - 16));

const phase = ref<'gust' | 'settle' | 'revealed'>('gust');
const pick = ref<ShelfBook | null>(null);
const drawn = ref<{ book: ShelfBook; others: ShelfBook[]; slot: number } | null>(null);
// Bumped whenever a run is superseded — by another gust, or by unmounting.
let run = 0;

/** As many neighbours as fit on one board, with the drawn book among them. */
const strip = computed<ShelfBook[]>(() => {
  const d = drawn.value;
  if (!d || boardWidth.value <= 0) return [];
  let used = spineWidth(d.book.title, d.book.page_count);
  const shown: ShelfBook[] = [];
  for (const b of d.others) {
    const needed = spineWidth(b.title, b.page_count) + SPINE_GAP;
    if (used + needed > boardWidth.value) break;
    used += needed;
    shown.push(b);
  }
  shown.splice(d.slot % (shown.length + 1), 0, d.book);
  return shown;
});

/**
 * Resolves once every animation and transition running on the shelf has ended,
 * so the phases hand over to each other instead of being timed to match the
 * durations in ShelfBoardView. Cancelled animations reject, hence `allSettled`.
 */
const shelfSettled = async () => {
  await nextTick();
  // A class only turns into a running animation once the browser has recalculated
  // styles, which happens after the next frame's rAF callbacks — asking any
  // earlier returns an empty list and the phase would end the moment it began.
  await new Promise(requestAnimationFrame);
  await new Promise(requestAnimationFrame);
  const element = boardViewRef.value?.$el as Element | undefined;
  if (!element) {
    return;
  }
  await Promise.allSettled(element.getAnimations({subtree: true}).map((animation) => animation.finished));
};

const blow = async () => {
  const pool = pick.value && props.books.length > 1
      ? props.books.filter((b) => b.id !== pick.value!.id)
      : props.books;
  const next = pool[Math.floor(Math.random() * pool.length)] ?? null;
  if (!next) return;

  // A random stretch of the shelf around the drawn book.
  const others = props.books.filter((b) => b.id !== next.id).sort(() => Math.random() - 0.5);
  drawn.value = {book: next, others: others.slice(0, CANDIDATES), slot: Math.floor(Math.random() * 1000)};

  const token = ++run;
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
    pick.value = next;
    phase.value = 'revealed';
    return;
  }

  // The breeze passes over the spines, then the one it settled on lights up,
  // and only once that has played out does the cover rise off the board.
  phase.value = 'gust';
  await shelfSettled();
  if (token !== run) {
    return;
  }
  pick.value = next;
  phase.value = 'settle';
  await shelfSettled();
  if (token !== run) {
    return;
  }
  phase.value = 'revealed';
};

const onKey = (e: KeyboardEvent) => e.key === 'Escape' && emit('close');

onMounted(() => {
  // Measure the board before drawing — the strip is packed to fit its width.
  setupObserver();
  blow();
  addEventListener('keydown', onKey);
});
onUnmounted(() => {
  run++;
  removeEventListener('keydown', onKey);
});
</script>

<style scoped>
/* Dim the shelf behind so the breeze owns the screen. */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgb(12 11 7 / 0.86);
  backdrop-filter: blur(10px);
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 220ms ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.picker {
  position: relative;
  z-index: 1;
  animation: fade-up 320ms ease both;
  width: min(560px, 100vw);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 22px;
  padding: 32px 20px;
}

/* Fixed slot so the board doesn't jump when the cover appears. */
.cover-slot {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  min-height: 288px;
}

.board {
  position: relative;
  width: 100%;
}

.reveal {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  text-align: center;
  max-width: 260px;
  /* Rises out of the gap it left on the board — held back just long enough for
     the spine to clear the shelf first. */
  animation: lift 900ms cubic-bezier(0.16, 0.9, 0.3, 1) 200ms both;
}

.reveal :deep(.cover) {
  box-shadow: var(--shadow-pop), 0 0 46px rgb(147 196 86 / 0.22);
}

.reveal-text {
  animation: fade-up 500ms ease 580ms both;
}

@keyframes lift {
  from {
    opacity: 0;
    transform: translateY(210px) rotate(-7deg) scale(0.72);
  }
  60% {
    opacity: 1;
  }
  to {
    opacity: 1;
    transform: translateY(0) rotate(0deg) scale(1);
  }
}

@keyframes fade-up {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* --- drifting leaves --- */
.leaves {
  position: absolute;
  inset: -16px 0;
  overflow: hidden;
  pointer-events: none;
}

.leaf {
  position: absolute;
  left: -40px;
  width: 30px;
  height: 31px;
  fill: var(--color-green-deep);
  animation: drift 1.7s ease-in both;
}

@keyframes drift {
  0% {
    opacity: 0;
    transform: translate(0, 0) rotate(-20deg) scale(var(--s));
  }
  15% {
    opacity: 0.9;
  }
  80% {
    opacity: 0.7;
  }
  100% {
    opacity: 0;
    transform: translate(640px, 26px) rotate(320deg) scale(var(--s));
  }
}

@media (prefers-reduced-motion: reduce) {
  .leaves, .reveal, .reveal-text, .picker {
    animation: none;
  }

  .leaves {
    display: none;
  }
}
</style>
