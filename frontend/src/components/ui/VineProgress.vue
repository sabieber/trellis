<!-- The signature reading-progress vine.
     A stem that fills with green and sprouts leaves as progress grows.
     Use for reading progress; use TBar for everything else.

     The stem is drawn in pixel space (viewBox tracks the measured width)
     so the leaves keep their shape at any width — stretching a fixed
     viewBox would smear them on wide cards. Leaf count scales with width;
     each leaf turns green once progress passes its position on the stem. -->
<template>
  <div ref="wrap" class="vine-progress" :style="{ height: height + 'px' }">
    <svg v-if="width > 0" :viewBox="`0 0 ${width} ${height}`">
      <path class="stem" pathLength="100" :d="stemD"></path>
      <path
          v-if="clamped > 0"
          class="grown"
          pathLength="100"
          :style="{ strokeDasharray: 100, strokeDashoffset: 100 - clamped }"
          :d="stemD"
      ></path>
      <path
          v-for="(leaf, i) in leaves"
          :key="i"
          class="leaf"
          :d="LEAF"
          :transform="leaf.tf"
          :fill="leaf.grown ? '#93c456' : '#38321f'"
      ></path>
    </svg>
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref} from 'vue';

const props = withDefaults(
    defineProps<{
      pct: number; // 0–100
      height?: number;
    }>(),
    {height: 20},
);

// Canonical stem wave from the design prototype, in a 100×18 space.
// The S command of `M2 14 C 26 14 34 5 52 8 S 86 14 98 6` expands to a
// second cubic whose first control point reflects (34,5) about (52,8).
const SEG1: number[][] = [[2, 14], [26, 14], [34, 5], [52, 8]];
const SEG2: number[][] = [[52, 8], [70, 11], [86, 14], [98, 6]];

// Undistorted leaf: a plump round-bellied leaf attached at (0,0), tip
// up-and-out at roughly (18,-21) (~50° pitch). Mirrored for alternates.
const LEAF = 'M0 0 C 0 -12.6 7.2 -19.8 18 -20.7 C 20.7 -9.9 12.6 -0.9 0 0 Z';

const wrap = ref<HTMLElement | null>(null);
const width = ref(0);
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  if (!wrap.value) return;
  width.value = wrap.value.clientWidth;
  resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) width.value = entry.contentRect.width;
  });
  resizeObserver.observe(wrap.value);
});

onUnmounted(() => resizeObserver?.disconnect());

const clamped = computed(() => Math.max(0, Math.min(100, props.pct)));

// Scale the canonical control points to the measured pixel box.
const stemD = computed(() => {
  const sx = width.value / 100;
  const sy = props.height / 18;
  const p = (pt: number[]) => `${(pt[0] * sx).toFixed(1)} ${(pt[1] * sy).toFixed(1)}`;
  return `M${p(SEG1[0])} C ${p(SEG1[1])} ${p(SEG1[2])} ${p(SEG1[3])} C ${p(SEG2[1])} ${p(SEG2[2])} ${p(SEG2[3])}`;
});

// Sampled lookup of the stem so leaves attach to the actual curve.
const STEM_SAMPLES: number[][] = (() => {
  const bez = (s: number[][], t: number) =>
      [0, 1].map((axis) => {
        const [p0, c1, c2, p1] = s.map((pt) => pt[axis]);
        const u = 1 - t;
        return u * u * u * p0 + 3 * u * u * t * c1 + 3 * u * t * t * c2 + t * t * t * p1;
      });
  const pts: number[][] = [];
  for (const seg of [SEG1, SEG2]) {
    for (let i = 0; i <= 32; i++) pts.push(bez(seg, i / 32));
  }
  return pts;
})();

function stemYAt(x: number): number {
  let best = STEM_SAMPLES[0];
  for (const pt of STEM_SAMPLES) {
    if (Math.abs(pt[0] - x) < Math.abs(best[0] - x)) best = pt;
  }
  return best[1];
}

// Per-instance horizontal jitter (±7%) so no two vines look identical.
// Lives in component scope, so positions are stable across re-renders and
// the grown thresholds stay consistent with the rendered spots.
const JITTER = Array.from({length: 7}, () => Math.random() - 0.5);

const leaves = computed(() => {
  const w = width.value;
  const count = Math.min(7, Math.max(4, Math.round(w / 120)));
  const spacing = w / (count + 1);
  // Shrink leaves when they'd crowd each other on narrow vines (leaf ≈ 27px long).
  const fit = Math.min(1, (spacing * 0.55) / 27);
  const scale = Math.max(0.8, Math.min(1.1, props.height / 20)) * fit;
  return Array.from({length: count}, (_, i) => {
    // Spread from 12% to 88% so the first leaf sprouts early, like the
    // prototype's 16/37/58/79 placement, with per-vine jitter.
    const base = 12 + (76 * i) / (count - 1);
    const frac = Math.min(94, Math.max(6, base + JITTER[i] * 14));
    // Odd leaves hang below the stem (vertical flip), like the prototype.
    const flip = i % 2 ? -1 : 1;
    const px = (frac / 100) * w;
    const py = stemYAt(frac) * (props.height / 18);
    return {
      grown: frac <= clamped.value,
      tf: `translate(${px.toFixed(1)} ${py.toFixed(1)}) scale(${scale.toFixed(2)} ${(flip * scale).toFixed(2)})`,
    };
  });
});
</script>

<style scoped>
.vine-progress {
  position: relative;
}

.vine-progress svg {
  display: block;
  width: 100%;
  height: 100%;
  overflow: visible;
}

.stem {
  stroke: #38321f;
  stroke-width: 2.5;
  fill: none;
  stroke-linecap: round;
}

.grown {
  stroke: #93c456;
  stroke-width: 2.5;
  fill: none;
  stroke-linecap: round;
  transition: stroke-dashoffset 0.4s ease;
}

.leaf {
  transition: fill 0.3s ease;
}
</style>
