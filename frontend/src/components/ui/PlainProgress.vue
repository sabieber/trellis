<!-- Plain 7px progress bar (goals, anything non-reading). -->
<template>
  <div class="plain-progress"><i :style="{ width: clamped + '%' }"></i></div>
</template>

<script setup lang="ts">
import {computed} from 'vue';

const props = defineProps<{ pct: number }>();
const clamped = computed(() => Math.max(0, Math.min(100, props.pct)));
</script>

<style scoped>
.plain-progress {
  height: 7px;
  border-radius: 999px;
  background: #322d1e;
  overflow: hidden;
}

.plain-progress > i {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: #93c456;
  transition: width 0.4s ease;
}
</style>
