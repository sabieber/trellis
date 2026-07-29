<!-- The reading streak as a raised garden bed: one planter, seven plants.
     A day with logged reading puts a plant in the soil, a missed day leaves a
     bare mound, and days still to come sit as seeds. Plants that belong to the
     running streak grow tall enough to reach the trellis behind the bed.

     Drawn in a fixed 300×86 pixel space and scaled to the card up to the width
     it was drawn for, so the plants keep their proportions on a phone without
     turning into a mural on a desktop screen. Leaves are the same shape the
     vine progress bar uses, so both speak the same visual language. -->
<template>
  <div class="bg-surface border border-line rounded-md p-3.5">
    <!-- The card stretches like the other home sections, its contents do not.
         The bed is drawn in a fixed 300×86 space, so an uncapped width scales
         the whole planter up several times over on a desktop screen; capping
         the column also keeps the week chip next to the count instead of a
         screen's width away from it. -->
    <div class="w-full max-w-[420px] mx-auto flex flex-col gap-2.5">
      <div class="flex items-baseline justify-between gap-2.5">
        <div class="flex items-baseline gap-1.5 min-w-0">
          <span class="t-display text-[34px] leading-none" :class="{'text-faint': !currentDays}">
            {{ currentDays }}
          </span>
          <span class="t-meta truncate">{{ $t('home.streakDays', {best: longestDays}, currentDays) }}</span>
        </div>
        <span class="badge badge-sm flex-none">{{ $t('home.streakWeeks', currentWeeks) }}</span>
      </div>

      <svg class="block w-full h-auto" viewBox="0 0 300 86" aria-hidden="true">
        <defs>
          <!-- The arch the lattice lives in; also the shape of its frame. -->
          <clipPath id="sb-arch">
            <path :d="ARCH"/>
          </clipPath>
          <!-- The trellis dissolves towards the top instead of stopping at a
               hard edge, so the card keeps its airy feel. -->
          <linearGradient id="sb-fade" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#fff" stop-opacity="0"/>
            <stop offset="0.55" stop-color="#fff" stop-opacity="0.7"/>
            <stop offset="1" stop-color="#fff" stop-opacity="1"/>
          </linearGradient>
          <mask id="sb-fade-mask">
            <rect x="0" y="0" width="300" height="54" fill="url(#sb-fade)"/>
          </mask>
          <linearGradient id="sb-soil" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#4b3722"/>
            <stop offset="1" stop-color="#241a10"/>
          </linearGradient>
          <linearGradient id="sb-clay" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#9d5e40"/>
            <stop offset="0.45" stop-color="#7f4830"/>
            <stop offset="1" stop-color="#553022"/>
          </linearGradient>
          <linearGradient id="sb-rim" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#b06f4d"/>
            <stop offset="1" stop-color="#7d4530"/>
          </linearGradient>
          <radialGradient id="sb-glow">
            <stop offset="0" stop-color="var(--color-green)" stop-opacity="0.3"/>
            <stop offset="1" stop-color="var(--color-green)" stop-opacity="0"/>
          </radialGradient>
        </defs>

        <!-- the trellis the app is named after -->
        <g mask="url(#sb-fade-mask)">
          <g clip-path="url(#sb-arch)" class="lattice">
            <path v-for="(line, index) in lattice" :key="index" :d="line"/>
          </g>
          <path class="frame" :d="ARCH"/>
        </g>

        <!-- soil, then what grows in it -->
        <path
            d="M8 50 q16 -3 32 -1 q16 2 32 -2 q16 -3 32 1 q16 3 32 -1 q16 -3 32 1 q16 2 32 2 q16 -2 32 -2 q16 -2 32 1 q14 2 28 1 V60 H8 Z"
            fill="url(#sb-soil)"
        />
        <circle v-for="(speck, index) in grit" :key="index" :cx="speck.x" :cy="speck.y" :r="speck.r" class="grit"/>

        <template v-for="mark in marks" :key="mark.date">
          <ellipse v-if="mark.glow" :cx="mark.x" cy="40" rx="21" ry="20" fill="url(#sb-glow)"/>
          <!-- Two nested groups: the outer one animates today's plant out of the
               soil, the inner one carries the placement — a CSS transform on the
               same element would overwrite the attribute transform.
               Tilted a couple of degrees around its root, because a hand-planted
               row never comes out as a set of parallel lines. -->
          <g v-if="mark.plant" :class="{sprouting: sprouting && mark.today}">
            <g :transform="`translate(${mark.x} 0) rotate(${mark.tilt} 0 ${SOIL_LINE + 6})`">
              <path :d="mark.plant.stem" class="stem"/>
              <path
                  v-for="(leaf, index) in mark.plant.leaves"
                  :key="index"
                  :d="LEAF"
                  :transform="`translate(0 ${leaf.y}) scale(${(leaf.flip * leaf.scale).toFixed(2)} ${leaf.scale})`"
                  :fill="leaf.flip > 0 ? mark.tone : 'var(--color-green-deep)'"
              />
              <circle
                  v-for="(fruit, index) in mark.plant.fruits"
                  :key="`f${index}`"
                  :cx="fruit.dx"
                  :cy="fruit.y"
                  :r="fruit.r"
                  :fill="fruit.tone === 'accent' ? mark.tone : 'var(--color-green)'"
              />
            </g>
          </g>
          <!-- A missed day is a mound of turned soil, a day still to come is a
               pair of seeds lying in it — raised versus flat, so the two never
               read as the same thing. -->
          <template v-else-if="mark.state === 'miss'">
            <path :d="`M${mark.x - 9} 53 q9 -8 18 0 Z`" fill="#5e462b"/>
            <path :d="`M${mark.x - 7} 50.6 q7 -4.4 14 0`" class="mound-crest"/>
          </template>
          <template v-else>
            <ellipse :cx="mark.x - 4" cy="51" rx="1.9" ry="1.4" class="seed"/>
            <ellipse :cx="mark.x + 4" cy="53" rx="1.9" ry="1.4" class="seed"/>
          </template>
        </template>

        <!-- the planter, drawn over the plant bases so they root behind it -->
        <rect x="2" y="54" width="296" height="8" rx="2.5" fill="url(#sb-rim)"/>
        <path d="M6 62 H294 L289 80 Q288.4 82 286 82 H14 Q11.6 82 11 80 Z" fill="url(#sb-clay)"/>
        <path d="M6 62 H294 L289 80 Q288.4 82 286 82 H14 Q11.6 82 11 80 Z" class="clay-edge"/>
        <path d="M4 55.4 H296" class="rim-light"/>
        <ellipse cx="150" cy="84" rx="128" ry="2.5" fill="#000" opacity="0.35"/>

        <text
            v-for="mark in marks"
            :key="`l${mark.date}`"
            :x="mark.x"
            y="75"
            text-anchor="middle"
            class="daymark"
            :class="{'is-today': mark.today}"
        >{{ mark.letter }}
        </text>
      </svg>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, onUnmounted, ref, watch} from 'vue';
import moment from 'moment';
import {dateKey} from '@/utils/activityHeat';

/** A single day of the current week, as delivered by `/api/stats/streak`. */
export interface StreakDay {
  /** The day in `YYYY-MM-DD` format. */
  date: string;
  /** Whether reading progress was logged on that day. */
  read: boolean;
}

const props = defineProps<{
  currentDays: number;
  longestDays: number;
  currentWeeks: number;
  week: StreakDay[];
}>();

// Same leaf as the vine progress bar: attached at (0,0), tip up-and-out.
const LEAF = 'M0 0 C 0 -12.6 7.2 -19.8 18 -20.7 C 20.7 -9.9 12.6 -0.9 0 0 Z';

/** Where a plant meets the soil; the planter front starts just below it. */
const SOIL_LINE = 52;

/** Fixed grit, so the soil reads as soil instead of as a flat brown band. */
const GRIT = [[18, 3], [46, 8], [74, 2], [96, 10], [128, 5], [152, 9], [176, 3],
  [204, 8], [232, 4], [258, 9], [278, 6], [62, 12], [140, 13], [218, 12]];

const grit = GRIT.map(([x, offset], index) => ({x, y: 49 + offset * 0.55, r: index % 3 ? 0.8 : 1.2}));

/** Fixed lean per position, so no two neighbours stand at the same angle. */
const TILT = [-4, 2, -2, 4, -3, 3, -1];

/** The arch the trellis frame draws and the lattice is clipped to. */
const ARCH = 'M22 50 V18 Q22 6 38 6 H262 Q278 6 278 18 V50';

/** The lattice of the trellis, as criss-crossed diagonals inside the arch. */
const lattice = Array.from({length: 20}, (_, index) => {
  const x = 20 - 44 + index * 22;
  return `M${x} 50 L${x + 44} 2 M${x} 2 L${x + 44} 50`;
});

/**
 * The three plant shapes, cycled so the bed reads as a mixed planting rather
 * than as a bar chart. `tall` plants belong to the running streak and reach up
 * into the trellis.
 */
function plantShape(kind: number, tall: boolean) {
  const top = tall ? 14 : 26;

  if (kind === 0) {
    return {
      stem: `M0 ${SOIL_LINE + 6} V${top}`,
      leaves: [{y: 49, flip: 1, scale: 0.62}, {y: 41, flip: -1, scale: 0.56},
        {y: 34, flip: 1, scale: 0.46}, {y: 27, flip: -1, scale: 0.36}],
      fruits: [{dx: 0, y: top - 1, r: 3, tone: 'accent'}],
    };
  }
  if (kind === 1) {
    return {
      stem: `M0 ${SOIL_LINE + 6} V${top + 8} q0 -8 7 -9`,
      leaves: [{y: 49, flip: -1, scale: 0.6}, {y: 40, flip: 1, scale: 0.5}, {y: 31, flip: -1, scale: 0.4}],
      fruits: [{dx: 7, y: top - 2, r: 2.8, tone: 'accent'}, {dx: -4, y: top + 9, r: 2.2, tone: 'green'}],
    };
  }
  return {
    stem: `M0 ${SOIL_LINE + 6} V${top + 12} M0 ${SOIL_LINE - 4} q-9 -7 -11 -18 M0 ${SOIL_LINE - 4} q9 -7 11 -18`,
    leaves: [{y: 49, flip: 1, scale: 0.56}, {y: 42, flip: -1, scale: 0.52}, {y: 35, flip: 1, scale: 0.42}],
    fruits: [{dx: -11, y: top + 12, r: 2.8, tone: 'accent'}, {dx: 11, y: top + 12, r: 2.8, tone: 'green'}],
  };
}

/** Localized Mon–Sun initials, matching the heatmap axis. */
const letters = computed(() => Array.from({length: 7}, (_, i) => moment().isoWeekday(i + 1).format('dd').charAt(0)));

/** How long the sprouting animation runs; must match the `sprout` keyframes. */
const SPROUT_MS = 900;

// Today's plant grows out of the soil the moment the first reading of the day is
// logged. Only on that flip, though — a plant that was already there when the
// home screen loaded is simply standing in the bed.
const sprouting = ref(false);
let sproutTimer: number | undefined;

watch(
    () => props.week.find((day) => day.date === dateKey(new Date()))?.read ?? false,
    (read, readBefore) => {
      if (!read || readBefore) return;
      clearTimeout(sproutTimer);
      sprouting.value = true;
      sproutTimer = setTimeout(() => (sprouting.value = false), SPROUT_MS);
    },
);

onUnmounted(() => clearTimeout(sproutTimer));

const marks = computed(() => {
  const today = dateKey(new Date());
  const step = 288 / 7;

  // The streak reaches back `currentDays - 1` days from its most recent day,
  // which is today when it was already used and yesterday when it was not.
  const last = props.week.find((day) => day.date === today)?.read
      ? moment(today)
      : moment(today).subtract(1, 'day');
  const streakStart = last.clone().subtract(Math.max(0, props.currentDays - 1), 'days').format('YYYY-MM-DD');

  return props.week.map((day, index) => {
    const x = Math.round(6 + step * (index + 0.5));
    const isToday = day.date === today;
    const state = day.read ? 'read' : (day.date > today ? 'future' : 'miss');
    const inStreak = day.read && props.currentDays > 0 && day.date >= streakStart;

    return {
      date: day.date,
      letter: letters.value[index],
      x,
      state,
      today: isToday,
      tilt: TILT[index],
      glow: isToday && day.read,
      tone: isToday || index % 2 === 0 ? 'var(--color-green-soft)' : 'var(--color-green)',
      plant: day.read ? plantShape(index % 3, inStreak) : null,
    };
  });
});
</script>

<style scoped>
.lattice path {
  stroke: var(--color-line);
  stroke-width: 1;
  fill: none;
  stroke-linecap: round;
  opacity: 0.85;
}

.frame {
  stroke: var(--color-line);
  stroke-width: 1.6;
  fill: none;
  stroke-linecap: round;
}

.grit {
  fill: #6d5335;
  opacity: 0.4;
}

/* Pushes up out of the soil, overshoots, and settles. The plant's own box is
   the reference, whose bottom centre is exactly where the stem meets the soil. */
.sprouting {
  transform-box: fill-box;
  transform-origin: 50% 100%;
  animation: sprout 900ms cubic-bezier(0.45, 0.05, 0.3, 1) both;
}

@keyframes sprout {
  0% {
    transform: scale(0.4, 0.02);
    opacity: 0;
  }
  35% {
    opacity: 1;
  }
  65% {
    transform: scale(1.08, 1.1) rotate(-2.5deg);
  }
  85% {
    transform: scale(0.99, 0.98) rotate(1.5deg);
  }
  100% {
    transform: scale(1, 1) rotate(0deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .sprouting {
    animation: none;
  }
}

.stem {
  stroke: var(--color-green-deep);
  stroke-width: 2.4;
  fill: none;
  stroke-linecap: round;
}

/* Catches the light on the crown of a turned-over mound. */
.mound-crest {
  stroke: #6d5335;
  stroke-width: 1.2;
  fill: none;
  stroke-linecap: round;
  opacity: 0.7;
}

.seed {
  fill: var(--color-green-deep);
  opacity: 0.45;
}

.clay-edge {
  fill: none;
  stroke: #c08a6b;
  stroke-width: 0.8;
  opacity: 0.35;
}

.rim-light {
  stroke: #c9906d;
  stroke-width: 1;
  opacity: 0.5;
  stroke-linecap: round;
}

/* Stamped into the clay: a dark cut with a lit edge underneath it. */
.daymark {
  font-family: var(--font-sans), sans-serif;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.08em;
  fill: #4a2a1c;
  paint-order: stroke;
  stroke: #c08a6b;
  stroke-width: 0.6;
  stroke-opacity: 0.28;
}

.daymark.is-today {
  fill: var(--color-green-soft);
  stroke: #2c1c10;
  stroke-opacity: 0.5;
}
</style>
