<!-- Inline value editor: shows a value with a small pen icon; clicking the pen
     turns the value into an input. Enter/blur commits, Escape cancels.
     `save` is a function prop (not an emit) so the component can await the
     parent's persistence and show a green checkmark on success. Return true
     from `save` to confirm, false to skip the feedback (e.g. after showing an
     error toast). -->
<template>
  <span class="inline-edit">
    <template v-if="!editing">
      <span>{{ displayText }}</span>
      <span v-if="saved" class="inline-edit__saved" aria-hidden="true">
        <CheckIcon class="size-3.5"/>
      </span>
      <button v-else type="button" class="inline-edit__trigger" :aria-label="label" @click="startEditing">
        <PencilIcon class="size-3.5"/>
      </button>
    </template>
    <input
        v-else
        ref="inputEl"
        v-model="draft"
        class="inline-edit__input"
        :type="type"
        :inputmode="type === 'number' ? 'numeric' : undefined"
        :style="{width: Math.max(draftText.length + 2, 8) + 'ch'}"
        :aria-label="label"
        @keydown.enter.prevent="commit"
        @keydown.esc.prevent="cancel"
        @blur="commit"
    />
  </span>
</template>

<script setup lang="ts">
import {computed, nextTick, onUnmounted, ref} from 'vue';
import {CheckIcon, PencilIcon} from '@heroicons/vue/24/outline';

const props = withDefaults(
    defineProps<{
      value: string | number | null;
      save: (value: string) => boolean | Promise<boolean>; // true = show success feedback
      type?: 'text' | 'number';
      label?: string; // aria-label for trigger and input
      placeholder?: string; // shown when value is empty
      validate?: (value: string) => boolean; // returning false keeps edit mode open
    }>(),
    {type: 'text', label: 'Edit value', placeholder: '—', validate: undefined},
);

const editing = ref(false);
// v-model on type="number" inputs auto-casts to a number, so draft may be
// either; draftText normalizes it for length/trim operations.
const draft = ref<string | number>('');
const draftText = computed(() => String(draft.value));
const inputEl = ref<HTMLInputElement | null>(null);
const saved = ref(false);
let savedTimer: ReturnType<typeof setTimeout> | undefined;

const displayText = computed(() =>
    props.value === null || props.value === '' ? props.placeholder : String(props.value));

const startEditing = async () => {
  saved.value = false;
  clearTimeout(savedTimer);
  draft.value = props.value === null ? '' : String(props.value);
  editing.value = true;
  await nextTick();
  inputEl.value?.focus();
  inputEl.value?.select();
};

const commit = async () => {
  if (!editing.value) return; // blur fired after Escape already cancelled
  const value = draftText.value.trim();
  if (props.validate && !props.validate(value)) return;
  editing.value = false;
  if (value === (props.value === null ? '' : String(props.value))) return;
  if (await props.save(value)) {
    saved.value = true;
    clearTimeout(savedTimer);
    savedTimer = setTimeout(() => (saved.value = false), 1500);
  }
};

const cancel = () => {
  editing.value = false;
};

onUnmounted(() => clearTimeout(savedTimer));
</script>

<style scoped>
.inline-edit {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.inline-edit__trigger {
  display: inline-flex;
  align-items: center;
  padding: 2px;
  border-radius: 4px;
  color: var(--color-faint);
  cursor: pointer;
  transition: color 0.15s;
}

.inline-edit__trigger:hover {
  color: var(--color-ink);
}

.inline-edit__saved {
  display: inline-flex;
  align-items: center;
  padding: 2px;
  color: var(--color-green-soft);
  animation: inline-edit-pop 0.18s ease;
}

@keyframes inline-edit-pop {
  from {
    transform: scale(0.6);
    opacity: 0;
  }
}

.inline-edit__input {
  font: inherit;
  color: inherit;
  letter-spacing: inherit;
  /* Match the display-mode line height (text at 1.5em line-height, or the
     18px pen button, whichever is taller) so the row doesn't grow in edit
     mode. 1em + 7px == 1.5em at 14px and == 18px (the button) at ~11px. */
  height: calc(1em + 7px);
  background: var(--color-surface-2);
  border: 1px solid var(--color-line);
  border-radius: var(--radius-xs);
  padding: 0 7px;
  outline: none;
}

.inline-edit__input:focus {
  border-color: var(--color-green-deep);
}

/* hide native spinners on number inputs */
.inline-edit__input::-webkit-outer-spin-button,
.inline-edit__input::-webkit-inner-spin-button {
  -webkit-appearance: none;
}

.inline-edit__input[type='number'] {
  -moz-appearance: textfield;
  appearance: textfield;
}
</style>
