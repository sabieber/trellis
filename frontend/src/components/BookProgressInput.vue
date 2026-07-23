<template>
  <div class="flex flex-col gap-1.5">
    <svg
        :viewBox="`0 0 ${W} ${H}`"
        class="w-full select-none text-base-content"
        style="touch-action: none"
        :style="{ cursor: dragging ? 'grabbing' : 'grab' }"
        @wheel.prevent="onWheel"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
    >
      <!-- covers hugging the page block, corners curling up; spine nub below -->
      <path :d="coverPath" stroke="currentColor" stroke-width="3" stroke-linecap="round"
            fill="none"/>
      <!-- pages, all pinched into the spine point -->
      <path v-for="(d, i) in pagePaths" :key="i" :d="d" stroke="currentColor" fill="none"
            stroke-width="1.1" stroke-opacity="0.5" stroke-linecap="round"/>
      <path v-for="(d, i) in topPagePaths" :key="'t' + i" :d="d" stroke="currentColor" fill="none"
            stroke-width="1.5" stroke-linecap="round"/>
      <!-- pages mid-flip, arcing over the spine -->
      <path v-for="(d, i) in flipPaths" :key="'f' + i" :d="d" stroke="currentColor" fill="none"
            stroke-width="1.4" stroke-linecap="round"/>
    </svg>
    <div class="flex items-baseline justify-center gap-1.5">
      <input
          type="number"
          class="input input-sm w-24 text-center"
          :value="current"
          min="0"
          :max="totalPages"
          required
          @focus="($event.target as HTMLInputElement).select()"
          @input="onInput"
          @blur="onBlur"
      />
      <span class="t-meta">/ {{ totalPages }}</span>
    </div>
    <p class="t-meta text-center opacity-60">Drag or scroll on the book to flip pages</p>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, onBeforeUnmount, ref, watch } from 'vue';

// --- canvas ---
const W = 260; // viewBox width
const H = 100; // viewBox height
const CX = 130; // x of the spine center
const BASE = 88; // y of the page baseline; covers and spine nub sit below it

// --- interaction ---
const PX_PER_PAGE = 6; // base drag sensitivity: px per page at slow speeds
const WHEEL_PER_PAGE = 100; // wheel delta per page — one wheel notch ≈ one page
const HAPTIC_TICK_MS = 10; // vibration pulse when a drag crosses a page
// pointer acceleration: faster drags shrink the distance between pages, so a
// quick swipe covers ground and a slow drag hones in on one page
const FINE_SPEED = 0.5; // px/ms up to which base sensitivity applies
const COARSE_SPEED = 2.5; // px/ms at which max speedup is reached
const MAX_SPEEDUP = 6; // sensitivity multiplier at COARSE_SPEED

// --- page stacks ---
const PAGE_COUNT = 26; // drawn page curves, not real pages
const PAGE_REACH = 98; // horizontal length of the bottom (longest) page
const REACH_FALLOFF = 0.9; // how much shorter each page above is (fore-edge slope)
const STACK_RISE = 1.15; // how much higher each page's fore-edge sits (stack thickness)
const ARCH_RISE = 1.3; // how much more each page arches up near the spine

// --- covers ---
const COVER_LENGTH = 106; // covers extend slightly past the pages
const COVER_DROOP = 5; // how far the cover tips sag below their spine end

// --- flip animation ---
const FLIP_S = 0.35; // seconds for a lone flip; crowded air clears faster
const MAX_AIRBORNE = 4; // max pages mid-flip at once; extra changes skip the visual
const AIRBORNE_SPEEDUP = 0.9; // extra flip speed per additional airborne page
const FLIP_APEX = 96; // height of the (virtual) upright pose a flip blends through

type Side = -1 | 1;
type PagePoints = {
  sx: number; sy: number;
  c1x: number; c1y: number;
  c2x: number; c2y: number;
  ex: number; ey: number;
};

// every page starts at the same spine pinch point and drapes onto its stack;
// higher pages (larger j) are shorter, sit higher and arch more
const pagePoints = (side: Side, j: number): PagePoints => {
  const reach = PAGE_REACH - j * REACH_FALLOFF;
  return {
    sx: CX, sy: BASE + 2 - j * 0.08,
    c1x: CX + side * 10, c1y: BASE - 12 - j * ARCH_RISE,
    c2x: CX + side * reach * 0.5, c2y: BASE - 5 - j * (STACK_RISE + 0.8),
    ex: CX + side * reach, ey: BASE - 1 - j * STACK_RISE,
  };
};

// virtual "standing upright over the spine" pose a flip passes through;
// exaggerated height because it only contributes half its weight mid-blend
const standingPoints = (dir: Side): PagePoints => ({
  sx: CX, sy: BASE + 2,
  c1x: CX + dir * 2, c1y: BASE - FLIP_APEX * 0.375,
  c2x: CX + dir * 18, c2y: BASE - FLIP_APEX * 0.71,
  ex: CX + dir * 10, ey: BASE - FLIP_APEX,
});

const toPath = (p: PagePoints) =>
    `M ${p.sx} ${p.sy} C ${p.c1x} ${p.c1y} ${p.c2x} ${p.c2y} ${p.ex} ${p.ey}`;

// quadratic bezier blend: exactly A at t=0, exactly B at t=1, pulled through M
const q = (a: number, m: number, b: number, t: number) =>
    (1 - t) * (1 - t) * a + 2 * t * (1 - t) * m + t * t * b;
const blendPoints = (a: PagePoints, m: PagePoints, b: PagePoints, t: number): PagePoints => {
  const out = {} as PagePoints;
  for (const k of Object.keys(a) as (keyof PagePoints)[]) out[k] = q(a[k], m[k], b[k], t);
  return out;
};

export default defineComponent({
  props: {
    modelValue: { type: Number, required: true },
    totalPages: { type: Number, required: true },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const dragging = ref(false);
    let dragPos = 0; // float page position while dragging (current is rounded)
    let lastX = 0;
    let lastT = 0;
    let smoothSpeed = 0;
    let wheelAcc = 0;

    // local copy so rapid wheel/drag events don't read a stale prop while
    // the emit -> parent -> prop round-trip is still in flight
    const current = ref(props.modelValue);
    watch(() => props.modelValue, (v) => { current.value = v; });

    const clamp = (v: number) => Math.min(props.totalPages, Math.max(0, Math.round(v)));
    const set = (v: number) => {
      const next = clamp(v);
      if (next !== current.value) {
        current.value = next;
        emit('update:modelValue', next);
      }
    };

    // how many of the drawn pages sit left (read) vs right (unread)
    const leftCount = computed(() => {
      const frac = current.value / props.totalPages;
      let k = Math.round(frac * PAGE_COUNT);
      if (current.value > 0) k = Math.max(k, 1);
      if (current.value < props.totalPages) k = Math.min(k, PAGE_COUNT - 1);
      return Math.min(PAGE_COUNT, Math.max(0, k));
    });

    const sidePaths = (side: Side, count: number) =>
        Array.from({ length: count }, (_, j) => toPath(pagePoints(side, j)));

    const pagePaths = computed(() => [
      ...sidePaths(-1, leftCount.value),
      ...sidePaths(1, PAGE_COUNT - leftCount.value),
    ]);

    // topmost page of each stack drawn stronger
    const topPagePaths = computed(() => {
      const paths: string[] = [];
      if (leftCount.value > 0) paths.push(toPath(pagePoints(-1, leftCount.value - 1)));
      if (leftCount.value < PAGE_COUNT) {
        paths.push(toPath(pagePoints(1, PAGE_COUNT - leftCount.value - 1)));
      }
      return paths;
    });

    const coverPath =
        `M ${CX - 2} ${BASE + 3} Q ${CX - 70} ${BASE + 3} ${CX - COVER_LENGTH} ${BASE + 3 + COVER_DROOP}` +
        ` M ${CX + 2} ${BASE + 3} Q ${CX + 70} ${BASE + 3} ${CX + COVER_LENGTH} ${BASE + 3 + COVER_DROOP}` +
        ` M ${CX - 5} ${BASE + 3} Q ${CX} ${BASE + 8} ${CX + 5} ${BASE + 3}`;

    // --- flip animation: pages arcing over the spine on value change ---
    // each flip blends from the actual top page of its source stack, through
    // an upright pose, onto the actual top page of the destination stack, so
    // it lifts off and lands on top of the stacks instead of crossing them
    const flips = ref<Array<{ dir: Side; t: number }>>([]);
    let raf = 0;
    let lastTs = 0;
    const reducedMotion =
        typeof window !== 'undefined' &&
        window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    const tick = (now: number) => {
      const dt = Math.min(0.05, (now - lastTs) / 1000);
      lastTs = now;
      // crowded air flips faster, so the animation keeps up with fast
      // scrolling and drains immediately once input stops
      const speed = (1 + (flips.value.length - 1) * AIRBORNE_SPEEDUP) / FLIP_S;
      const next = flips.value
          .map((f) => ({ ...f, t: f.t + dt * speed }))
          .filter((f) => f.t < 1);
      flips.value = next;
      raf = next.length ? requestAnimationFrame(tick) : 0;
    };

    const spawnFlip = (dir: Side) => {
      if (reducedMotion || flips.value.length >= MAX_AIRBORNE) return;
      flips.value = [...flips.value, { dir, t: 0 }];
      if (!raf) {
        lastTs = performance.now();
        raf = requestAnimationFrame(tick);
      }
    };

    watch(current, (nv, ov) => spawnFlip(nv > ov ? 1 : -1));
    onBeforeUnmount(() => cancelAnimationFrame(raf));

    const flipPaths = computed(() => {
      const jLeft = Math.max(0, leftCount.value - 1);
      const jRight = Math.max(0, PAGE_COUNT - leftCount.value - 1);
      return flips.value.map((f) => {
        const from = pagePoints(f.dir === 1 ? 1 : -1, f.dir === 1 ? jRight : jLeft);
        const to = pagePoints(f.dir === 1 ? -1 : 1, f.dir === 1 ? jLeft : jRight);
        const ease = f.t * f.t * (3 - 2 * f.t); // smoothstep
        return toPath(blendPoints(from, standingPoints(f.dir), to, ease));
      });
    });

    const onWheel = (e: WheelEvent) => {
      // deltaMode 1 = lines (Firefox wheel), roughly 3 per notch vs ~100 px
      wheelAcc += e.deltaMode === 1 ? e.deltaY * 34 : e.deltaY;
      const pages = Math.trunc(wheelAcc / WHEEL_PER_PAGE);
      if (pages !== 0) {
        wheelAcc -= pages * WHEEL_PER_PAGE;
        set(current.value + pages);
      }
    };

    // haptic tick per page crossed while dragging, so users can feel where
    // they land; no-ops where vibration is unsupported (e.g. iOS Safari)
    const vibrateTick = () => navigator.vibrate?.(HAPTIC_TICK_MS);

    const onPointerDown = (e: PointerEvent) => {
      dragging.value = true;
      dragPos = current.value;
      lastX = e.clientX;
      lastT = e.timeStamp;
      smoothSpeed = 0;
      (e.currentTarget as Element).setPointerCapture(e.pointerId);
    };
    const onPointerMove = (e: PointerEvent) => {
      if (!dragging.value) return;
      const dx = e.clientX - lastX;
      const dt = e.timeStamp - lastT;
      lastX = e.clientX;
      lastT = e.timeStamp;
      // smoothed drag speed drives the acceleration between FINE and COARSE
      if (dt > 0) smoothSpeed = smoothSpeed * 0.7 + (Math.abs(dx) / dt) * 0.3;
      const t = Math.min(1, Math.max(0, (smoothSpeed - FINE_SPEED) / (COARSE_SPEED - FINE_SPEED)));
      const speedup = 1 + (MAX_SPEEDUP - 1) * t;
      // swipe left = forward, like grabbing a page and turning it
      const before = current.value;
      // clamp the float position too, so overshooting past the ends leaves no
      // dead zone to drag back out of
      dragPos = Math.min(props.totalPages, Math.max(0, dragPos - (dx / PX_PER_PAGE) * speedup));
      set(dragPos);
      if (current.value !== before) vibrateTick();
    };
    const onPointerUp = () => {
      dragging.value = false;
    };

    const onInput = (e: Event) => {
      const raw = (e.target as HTMLInputElement).value;
      if (raw === '') return; // wait for blur
      const v = Number(raw);
      if (Number.isFinite(v)) set(v);
    };
    const onBlur = (e: Event) => {
      // restore display when the field was left empty or out of range
      (e.target as HTMLInputElement).value = String(current.value);
    };

    return {
      W, H,
      current, dragging, pagePaths, topPagePaths, coverPath, flipPaths,
      onWheel, onPointerDown, onPointerMove, onPointerUp, onInput, onBlur,
    };
  },
});
</script>
