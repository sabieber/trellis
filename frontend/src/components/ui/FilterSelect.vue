<!-- A single-select you can type in: the options narrow as you type, which a
     native <select> cannot do. Use it where the list is long enough to search;
     a handful of fixed options is still better served by a plain <select>.

     Selecting nothing is a real choice here, so the list always starts with the
     "all" entry, which clears the filter. -->
<template>
  <div class="dropdown w-full sm:w-44">
    <div class="relative">
      <input
          ref="inputEl"
          class="input input-sm w-full pr-7"
          type="text"
          role="combobox"
          autocomplete="off"
          :aria-label="label"
          :aria-expanded="open"
          :aria-controls="listId"
          :aria-activedescendant="highlighted >= 0 ? `${listId}-${highlighted}` : undefined"
          :value="open ? query : selectedLabel"
          :placeholder="allLabel"
          @focus="onFocus"
          @blur="open = false"
          @input="onInput"
          @keydown.enter.prevent="commitHighlighted"
          @keydown.esc.prevent="inputEl?.blur()"
          @keydown.down.prevent="move(1)"
          @keydown.up.prevent="move(-1)"
      />
      <ChevronDownIcon class="size-4 absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none text-faint"/>
    </div>

    <!-- Rendered only while open: daisyUI hides the closed dropdown with CSS
         alone, which would mount an anchor per option — hundreds of them, times
         one dropdown per filter — before the user opens anything.

         No minimum width on a phone: the filters sit in a two-column grid there,
         and a list wider than its column pushes the whole page sideways. -->
    <ul
        v-if="open"
        :id="listId"
        role="listbox"
        class="dropdown-content menu bg-base-100 rounded-box z-1 w-full sm:min-w-52 p-2 shadow-sm mt-1 max-h-72 flex-nowrap overflow-y-auto"
    >
      <li v-for="(option, index) in matches" :key="option.value">
        <!-- Commit on mousedown with the default prevented: a click handler
             would race the input's blur, which closes the dropdown before the
             click lands. Do not "simplify" this to @click. -->
        <a
            :id="`${listId}-${index}`"
            role="option"
            :aria-selected="option.value === modelValue"
            class="whitespace-normal"
            :class="{ 'menu-active': index === highlighted, 'text-faint': !option.value }"
            @mousedown.prevent="commit(option.value)"
        >{{ option.label }}</a>
      </li>
      <li v-if="!matches.length" class="t-meta px-3 py-2">{{ noMatchText }}</li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import {computed, ref, useId} from 'vue';
import {ChevronDownIcon} from '@lucide/vue';

export interface FilterOption {
  value: string;
  label: string;
}

const props = defineProps<{
  modelValue: string; // '' means no filter
  options: FilterOption[];
  allLabel: string; // the entry that clears the filter, e.g. "All authors"
  label: string; // becomes the input's aria-label
  noMatchText: string;
}>();

const emit = defineEmits<{ 'update:modelValue': [string] }>();

const open = ref(false);
const query = ref('');
const highlighted = ref(-1);
const inputEl = ref<HTMLInputElement | null>(null);
const listId = useId();

const selectedLabel = computed(
    () => props.options.find((option) => option.value === props.modelValue)?.label ?? '');

// The "all" entry is part of the list, so clearing the filter works the same
// way as picking one — and it can be typed for as well.
const matches = computed(() => {
  const typed = query.value.trim().toLowerCase();
  return [{value: '', label: props.allLabel}, ...props.options]
      .filter((option) => option.label.toLowerCase().includes(typed));
});

const onFocus = () => {
  open.value = true;
  query.value = '';
  // Start on the current selection, so Enter without typing changes nothing.
  highlighted.value = matches.value.findIndex((option) => option.value === props.modelValue);
  inputEl.value?.select();
};

const onInput = (event: Event) => {
  query.value = (event.target as HTMLInputElement).value;
  highlighted.value = matches.value.length ? 0 : -1;
};

const commit = (value: string) => {
  emit('update:modelValue', value);
  query.value = '';
  inputEl.value?.blur();
};

const commitHighlighted = () => {
  const picked = matches.value[highlighted.value];
  if (picked) commit(picked.value);
};

const move = (delta: number) => {
  const count = matches.value.length;
  if (!count) return;
  highlighted.value = highlighted.value < 0
      ? (delta > 0 ? 0 : count - 1)
      : (highlighted.value + delta + count) % count;
};
</script>
