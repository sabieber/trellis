<!-- One labelled section of removable badges (genres or tags) plus an add
     control that suggests from the labels the user has already used.
     Presentational only: it emits `add`/`remove` and never calls the API, so
     persistence stays with the parent view. -->
<template>
  <section>
    <h2 class="t-eyebrow mb-2">{{ title }}</h2>
    <div class="flex flex-wrap items-center gap-2">
      <Chip v-for="label in labels" :key="label">
        {{ label }}
        <button
            type="button"
            class="label-remove"
            :aria-label="$t('bookDetail.removeLabel', { label })"
            @click="$emit('remove', label)"
        >
          <XIcon class="size-3"/>
        </button>
      </Chip>

      <span v-if="!labels.length && !adding" class="t-meta">{{ emptyText }}</span>

      <button
          v-if="!adding"
          type="button"
          class="label-add"
          :aria-label="addLabel"
          @click="startAdding"
      >
        <PlusIcon class="size-3.5"/>
      </button>

      <!-- daisyUI's dropdown opens and closes on :focus-within, so there is no
           open/close state to track here. -->
      <!-- A form, not a bare input: mobile keyboards do not reliably fire a
           keydown with key "Enter" (GBoard sends keyCode 229 while composing),
           but their return key does submit a single-input form. -->
      <form v-else class="dropdown" @submit.prevent="commitDraft">
        <input
            ref="inputEl"
            :value="draft"
            class="label-input"
            type="text"
            enterkeyhint="done"
            role="combobox"
            aria-autocomplete="list"
            :aria-label="addLabel"
            :aria-expanded="matches.length > 0"
            :aria-controls="listId"
            :aria-activedescendant="highlighted >= 0 ? `${listId}-${highlighted}` : undefined"
            :maxlength="MAX_LABEL_LENGTH"
            @input="onInput"
            @keydown.esc.prevent="stopAdding"
            @keydown.down.prevent="move(1)"
            @keydown.up.prevent="move(-1)"
            @blur="stopAdding"
        />
        <ul
            v-if="matches.length"
            :id="listId"
            role="listbox"
            class="dropdown-content menu bg-base-100 rounded-box z-1 w-52 p-2 shadow-sm mt-1 max-h-60 flex-nowrap overflow-y-auto"
        >
          <li v-for="(suggestion, index) in matches" :key="suggestion">
            <!-- Commit on mousedown with the default prevented: a click handler
                 would race the input's blur, which closes the dropdown before
                 the click lands. Do not "simplify" this to @click. -->
            <a
                :id="`${listId}-${index}`"
                role="option"
                :aria-selected="index === highlighted"
                :class="{ 'menu-active': index === highlighted }"
                @mousedown.prevent="commit(suggestion)"
            >{{ suggestion }}</a>
          </li>
        </ul>
      </form>
    </div>
  </section>
</template>

<script setup lang="ts">
import {computed, nextTick, ref, useId} from 'vue';
import {PlusIcon, XIcon} from '@lucide/vue';
import Chip from '@/components/ui/Chip.vue';

// Matches the backend's cap; the input just stops the user earlier.
const MAX_LABEL_LENGTH = 40;

const props = defineProps<{
  title: string; // already translated section heading
  labels: string[];
  suggestions: string[]; // every label of this kind the user has used before
  emptyText: string;
  addLabel: string; // aria-label for the add button and the input
}>();

const emit = defineEmits<{ add: [string]; remove: [string] }>();

const adding = ref(false);
const draft = ref('');
const highlighted = ref(-1);
const inputEl = ref<HTMLInputElement | null>(null);
const listId = useId();

const matches = computed(() => {
  const typed = draft.value.trim().toLowerCase();
  const taken = props.labels.map((label) => label.toLowerCase());
  return props.suggestions.filter(
      (s) => !taken.includes(s.toLowerCase()) && s.toLowerCase().includes(typed));
});

// Read the value off the event instead of using v-model: v-model suppresses
// updates while an IME is composing, and GBoard composes every word it
// predicts, so the draft — and with it the suggestions — froze on Android.
const onInput = (event: Event) => {
  draft.value = (event.target as HTMLInputElement).value;
  highlighted.value = -1;
};

const startAdding = async () => {
  adding.value = true;
  draft.value = '';
  highlighted.value = -1;
  await nextTick();
  inputEl.value?.focus();
};

const stopAdding = () => {
  adding.value = false;
};

const commit = (label: string) => {
  const value = label.trim();
  if (value) emit('add', value);
  draft.value = '';
  highlighted.value = -1;
  // Stay in add mode so several labels can be typed in a row.
  inputEl.value?.focus();
};

// Enter takes the highlighted suggestion if the user arrowed to one, otherwise
// whatever was typed — creating a label is just the "nothing matched" case.
const commitDraft = () => {
  const picked = highlighted.value >= 0 ? matches.value[highlighted.value] : undefined;
  commit(picked ?? draft.value);
};

const move = (delta: number) => {
  const count = matches.value.length;
  if (!count) return;
  highlighted.value = highlighted.value < 0
      ? (delta > 0 ? 0 : count - 1)
      : (highlighted.value + delta + count) % count;
};
</script>

<style scoped>
.label-remove {
  display: inline-flex;
  align-items: center;
  margin: -2px -4px -2px 0;
  padding: 2px;
  border-radius: 999px;
  color: var(--color-faint);
  cursor: pointer;
  transition: color 0.15s;
}

.label-remove:hover {
  color: var(--color-ink);
}

.label-add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  border-radius: 999px;
  border: 1px dashed var(--color-line);
  color: var(--color-faint);
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.label-add:hover {
  color: var(--color-ink);
  border-color: var(--color-green-deep);
}

.label-input {
  font-family: var(--font-sans), sans-serif;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-ink);
  width: 12rem;
  padding: 7px 13px;
  border-radius: 999px;
  border: 1px solid var(--color-line);
  background: var(--color-surface-2);
  outline: none;
}

.label-input:focus {
  border-color: var(--color-green-deep);
}
</style>
