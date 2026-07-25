<!-- Renders the shared chart tooltip near the pointer. Mounted once (in
     StatsView); every chart mark drives it via useChartTooltip().marks(). -->
<template>
  <Teleport to="body">
    <div v-if="state.visible" class="chart-tooltip" :style="tipStyle">{{ state.content }}</div>
  </Teleport>
</template>

<script lang="ts">
import {computed, defineComponent, onMounted, onUnmounted} from 'vue';
import {useChartTooltip} from '@/composables/useChartTooltip';

export default defineComponent({
  setup() {
    const {state, hide, isPinned} = useChartTooltip();

    // Keep the tooltip on-screen when the pointer is near a viewport edge.
    const tipStyle = computed(() => ({
      left: `${Math.min(Math.max(state.x, 96), window.innerWidth - 96)}px`,
      top: `${state.y}px`,
    }));

    // A pinned (touch) tooltip is dismissed by a tap anywhere but a mark —
    // marks stopPropagation, so any pointerdown reaching the document is "outside".
    const onDocPointerDown = () => {
      if (isPinned()) hide();
    };
    const onDismiss = () => hide();

    onMounted(() => {
      document.addEventListener('pointerdown', onDocPointerDown);
      window.addEventListener('scroll', onDismiss, true);
      window.addEventListener('resize', onDismiss);
    });
    onUnmounted(() => {
      document.removeEventListener('pointerdown', onDocPointerDown);
      window.removeEventListener('scroll', onDismiss, true);
      window.removeEventListener('resize', onDismiss);
    });

    return {state, tipStyle};
  },
});
</script>

<style scoped>
.chart-tooltip {
  position: fixed;
  z-index: 60;
  transform: translate(-50%, calc(-100% - 10px));
  pointer-events: none;
  max-width: 220px;
  padding: 6px 9px;
  border-radius: 6px;
  background: var(--color-surface-3);
  border: 1px solid var(--color-line);
  color: var(--color-ink);
  font-size: 12px;
  line-height: 1.35;
  box-shadow: 0 6px 18px rgb(0 0 0 / 0.4);
}
</style>
