<template>
  <div
      class="spine"
      :style="{
        width: spineWidth + 'px',
        height: actualHeight + 'px',
        background: colors.bg,
        color: colors.text,
      }"
  >
    <span class="spine-title">{{ title }}</span>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {
  colorwayForTitle,
  COLORWAY_COLORS,
  spineWidthForTitle,
  spineHeightOffset,
  type Colorway
} from '@/utils/bookColorway';

const props = withDefaults(
    defineProps<{
      title: string;
      author?: string;
      colorway?: Colorway | '';
      height?: number;
    }>(),
    {author: '', colorway: '', height: 200},
);

const spineWidth = computed(() => spineWidthForTitle(props.title));

const actualHeight = computed(() =>
    Math.round(props.height + spineHeightOffset(props.title))
);

const colors = computed(() => {
  const cw = colorwayForTitle(props.title, props.colorway);
  return COLORWAY_COLORS[cw];
});
</script>

<style scoped>
.spine {
  position: relative;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px 3px 1px 1px;
  overflow: hidden;
  cursor: pointer;
  box-shadow: inset 2px 0 0 rgb(255 255 255 / 0.08),
  inset -2px 0 0 rgb(0 0 0 / 0.18),
  0 -1px 2px rgb(0 0 0 / 0.15);
  font-family: var(--font-serif), serif;
  transition: transform 0.15s ease;
}

.spine:hover {
  transform: translateY(-6px);
}

.spine::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, rgb(255 255 255 / 0.06), transparent 30%, transparent 70%, rgb(0 0 0 / 0.12));
  pointer-events: none;
}

.spine-title {
  writing-mode: vertical-rl;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-height: calc(100% - 16px);
  opacity: 0.88;
}
</style>
