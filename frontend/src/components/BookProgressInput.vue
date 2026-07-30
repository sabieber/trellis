<template>
  <div class="flex flex-col gap-1.5">
    <svg
        :viewBox="`0 0 ${W} ${H}`"
        class="w-full select-none"
        style="touch-action: none"
        :style="{ cursor: dragging ? 'grabbing' : 'grab' }"
        @wheel.prevent="onWheel"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
    >
      <defs>
        <!-- Paper seen edge-on: lit along the top sheet, shaded down into the
             block. Fixed to user space so both stacks are shaded alike. -->
        <linearGradient id="bp-paper" gradientUnits="userSpaceOnUse"
                        :x1="0" :y1="BASE - 34" :x2="0" :y2="BASE + 3">
          <stop offset="0" stop-color="#e7ddc6"/>
          <stop offset="1" stop-color="#b3a687"/>
        </linearGradient>
        <!-- The same halo the vine puts on its growing tip, here on the spine:
             this is where reading is happening. -->
        <radialGradient id="bp-glow">
          <stop offset="0" stop-color="var(--color-green)" stop-opacity="0.2"/>
          <stop offset="1" stop-color="var(--color-green)" stop-opacity="0"/>
        </radialGradient>
        <!-- Leather, same warm clay family as the streak bed planter. -->
        <linearGradient id="bp-spine" gradientUnits="userSpaceOnUse"
                        :x1="0" :y1="BASE + 2" :x2="0" :y2="BASE + 10">
          <stop offset="0" stop-color="#6b4527"/>
          <stop offset="1" stop-color="#3a2415"/>
        </linearGradient>
      </defs>

      <ellipse :cx="CX" :cy="BASE + 10" rx="104" ry="2.2" fill="#000" opacity="0.35"/>
      <ellipse :cx="CX" :cy="BASE - 4" rx="26" ry="20" fill="url(#bp-glow)"/>

      <!-- the page stacks as solid blocks, with the single sheets on top -->
      <path v-for="(d, i) in blockPaths" :key="'b' + i" :d="d" fill="url(#bp-paper)"/>
      <path v-for="(d, i) in pagePaths" :key="i" :d="d" class="sheet" fill="none"/>
      <path v-for="(d, i) in topPagePaths" :key="'t' + i" :d="d" class="sheet-top" fill="none"/>
      <!-- pages mid-flip, arcing over the spine -->
      <path v-for="(d, i) in flipPaths" :key="'f' + i" :d="d" class="sheet-flip" fill="none"/>

      <!-- covers hugging the underside of the page block, running a little past
           the fore edge -->
      <path v-for="(d, i) in coverPaths" :key="'c' + i" :d="d" class="cover" fill="none"/>
      <path v-for="(d, i) in coverLights" :key="'cl' + i" :d="d" class="cover-light" fill="none"/>
      <!-- the leather wrapping the fold, taking over from the covers -->
      <path :d="spinePath" class="spine"/>
      <path :d="spineLight" class="spine-light" fill="none"/>
    </svg>
    <div class="flex justify-center">
      <div class="page-chip">
        <input
            type="number"
            class="page-field"
            :value="current"
            min="0"
            :max="totalPages"
            required
            @focus="($event.target as HTMLInputElement).select()"
            @input="onInput"
            @blur="onBlur"
        />
        <span class="t-meta">{{ mode === 'percentage' ? '%' : `/ ${totalPages}` }}</span>
      </div>
    </div>
    <p class="t-meta text-center opacity-60">{{ $t('progressModal.dragHint') }}</p>
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
const ARCH_RISE = 1.3; // how much more each page arches up near the fold;
// the lowest sheet does not arch at all — it lies flat on the cover board

// --- covers ---
const COVER_GAP = 2.1; // how far below the lowest sheet the cover runs
const COVER_OVERHANG = 5; // how far the cover reaches past the fore edge
const SPINE_HALF = 5.5; // half-width of the leather bound around the fold
const SPINE_FOOT = 5; // half-width where it meets the table
const SPINE_DEPTH = 7.6; // how far below the page baseline its foot sits

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

// every page starts at the same fold and drapes onto its stack;
// higher pages (larger j) are shorter, sit higher and arch more
const pagePoints = (side: Side, j: number): PagePoints => {
  const reach = PAGE_REACH - j * REACH_FALLOFF;
  return {
    sx: CX, sy: BASE + 2 - j * 0.08,
    c1x: CX + side * 10, c1y: BASE + 1 - j * (ARCH_RISE + 0.52),
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
    mode: { type: String, default: 'pages' },
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

    // The block of paper between the top sheet and the lowest one: the top
    // sheet's curve, out along the fore edge, and back underneath along the
    // lowest sheet — so a stack is only ever as deep as its own pages.
    const blockPath = (side: Side, count: number) => {
      if (count === 0) return null;
      const p = pagePoints(side, count - 1);
      const b = pagePoints(side, 0); // longest, lowest page: it is the underside
      return `${toPath(p)} L ${b.ex} ${b.ey} C ${b.c2x} ${b.c2y} ${b.c1x} ${b.c1y} ${b.sx} ${b.sy} Z`;
    };

    const blockPaths = computed(() =>
        [blockPath(-1, leftCount.value), blockPath(1, PAGE_COUNT - leftCount.value)]
            .filter((d): d is string => d !== null));

    // The cover follows the lowest sheet a hair below it and runs a little past
    // the fore edge, straight on along the curve's exit direction — a board is
    // stiff, it does not curl. Stroked rather than filled: a band hugging a
    // curve is that curve with a width. `dy` lifts the lit edge off the leather.
    const coverEdge = (side: Side, dy: number) => {
      const b = pagePoints(side, 0);
      const y = (v: number) => v + COVER_GAP + dy;
      const [tx, ty] = [b.ex - b.c2x, b.ey - b.c2y];
      const len = Math.hypot(tx, ty);
      return `M ${CX} ${y(b.sy)} C ${b.c1x} ${y(b.c1y)} ${b.c2x} ${y(b.c2y)} ${b.ex} ${y(b.ey)}` +
          ` l ${(tx / len * COVER_OVERHANG).toFixed(1)} ${(ty / len * COVER_OVERHANG).toFixed(1)}`;
    };
    const coverPaths = [coverEdge(-1, 0), coverEdge(1, 0)];
    const coverLights = [coverEdge(-1, -1.1), coverEdge(1, -1.1)];

    // The binding: where the two covers meet, the leather wraps a block with a
    // flat bottom the book stands on. Its top follows the dip of the covers.
    // The leather wrapping the fold, seen edge-on: it starts under the covers,
    // which hide its top, and tapers to the flat foot the book stands on.
    const spineTop = BASE + 3.5;
    const spineBottom = BASE + SPINE_DEPTH;
    const spinePath =
        `M ${CX - SPINE_HALF} ${spineTop} L ${CX + SPINE_HALF} ${spineTop}` +
        ` L ${CX + SPINE_FOOT} ${spineBottom - 1.4}` +
        ` Q ${CX + SPINE_FOOT} ${spineBottom} ${CX + SPINE_FOOT - 1.4} ${spineBottom}` +
        ` L ${CX - SPINE_FOOT + 1.4} ${spineBottom}` +
        ` Q ${CX - SPINE_FOOT} ${spineBottom} ${CX - SPINE_FOOT} ${spineBottom - 1.4} Z`;
    const spineLight =
        `M ${CX - SPINE_FOOT + 2.4} ${spineBottom - 1.1} L ${CX + SPINE_FOOT - 2.4} ${spineBottom - 1.1}`;

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
      W, H, CX, BASE,
      current, dragging, blockPaths, pagePaths, topPagePaths, coverPaths, coverLights, spinePath, spineLight, flipPaths,
      onWheel, onPointerDown, onPointerMove, onPointerUp, onInput, onBlur,
    };
  },
});
</script>

<style scoped>
/* Single sheets scored into the paper block, the top one catching the light. */
.sheet {
  stroke: #7d7157;
  stroke-width: 0.9;
  stroke-opacity: 0.3;
  stroke-linecap: round;
}

/* The sheet you are looking at, lit just enough to sit apart from the stack. */
.sheet-top {
  stroke: #f2ead6;
  stroke-width: 1;
  stroke-opacity: 0.85;
  stroke-linecap: round;
}

/* Airborne, so lit from every side and brighter than anything in the stack. */
.sheet-flip {
  stroke: #fbf6e8;
  stroke-width: 1.2;
  stroke-linecap: round;
}

/* Leather binding, same warm clay family as the streak bed planter. */
.cover {
  stroke: #4d3320;
  stroke-width: 3.2;
  stroke-linecap: round;
}

.spine {
  fill: url(#bp-spine);
}

.spine-light {
  stroke: #b07c4f;
  stroke-width: 0.8;
  stroke-opacity: 0.4;
  stroke-linecap: round;
}

.cover-light {
  stroke: #8a6140;
  stroke-width: 1.1;
  stroke-opacity: 0.5;
  stroke-linecap: round;
}

/* The page count as a stamped chip, like the streak bed's week badge. */
.page-chip {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  padding: 0.25rem 0.75rem;
  background: var(--color-surface-2);
  border: 1px solid var(--color-line);
  border-radius: var(--radius-sm);
}

.page-field {
  width: 3.25rem;
  background: none;
  border: none;
  outline: none;
  text-align: right;
  font-family: var(--font-serif), serif;
  font-weight: 600;
  font-size: 20px;
  line-height: 1.1;
  color: var(--color-green-soft);
  /* the native spinner fights the drag gesture for the same job and looks
     nothing like the rest of the card */
  appearance: textfield;
}

.page-field::-webkit-inner-spin-button,
.page-field::-webkit-outer-spin-button {
  appearance: none;
  margin: 0;
}
</style>
