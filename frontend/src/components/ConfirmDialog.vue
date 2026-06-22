<template>
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="t-title text-lg">{{ title }}</h3>
      <p class="text-sm text-ink-dim py-4">{{ message }}</p>
      <div class="modal-action">
        <Button variant="ghost" :class="confirmClass" @click="$emit('confirm')">{{ confirmLabel }}</Button>
        <Button variant="ghost" @click="$emit('cancel')">Cancel</Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
  </div>
</template>

<script lang="ts">
import {defineComponent} from 'vue';
import Button from '@/components/ui/Button.vue';

export default defineComponent({
  components: {Button},
  props: {
    title: {type: String, required: true},
    message: {type: String, required: true},
    confirmLabel: {type: String, default: 'Delete'},
    confirmClass: {type: String, default: 'text-[#c47556]!'},
  },
  emits: ['confirm', 'cancel'],
});
</script>
