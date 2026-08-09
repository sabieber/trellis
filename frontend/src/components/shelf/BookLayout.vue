<!-- Renders a list of books in the selected layout mode and measures itself, so
     every view that shows books does it at the same sizes. Pair it with
     `LayoutModeSelect`, which picks the mode. -->
<template>
  <div ref="contentRef">
    <ShelfListView
        v-if="mode === 'list'"
        :books="books"
        :cover-width="listCoverWidth"
        :removable="removable"
        :date-label="dateLabel"
        :date-field="dateField"
        @remove-book="$emit('removeBook', $event)"
    />
    <ShelfGridView
        v-else-if="mode === 'grid'"
        :books="books"
        :tile-width="gridTileWidth"
    />
    <ShelfBoardView
        v-else-if="mode === 'shelf'"
        :books="books"
        :spine-height="spineHeight"
        :container-width="containerWidth"
    />
    <ShelfPileView
        v-else
        :books="books"
    />
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, ref} from 'vue';
import ShelfListView from '@/components/shelf/ShelfListView.vue';
import ShelfGridView from '@/components/shelf/ShelfGridView.vue';
import ShelfBoardView from '@/components/shelf/ShelfBoardView.vue';
import ShelfPileView from '@/components/shelf/ShelfPileView.vue';
import {useContainerWidth} from '@/composables/useContainerWidth';
import type {LayoutMode} from '@/composables/useLayoutMode';
import type {ShelfBook} from '@/types/shelf';

const MD_BREAKPOINT = 768;
const LIST_COVER_SM = 56;
const LIST_COVER_LG = 72;
const GRID_TILE_SM = 80;
const GRID_TILE_LG = 112;
const SPINE_HEIGHT_SM = 160;
const SPINE_HEIGHT_LG = 200;

withDefaults(defineProps<{
  books: ShelfBook[];
  mode: LayoutMode;
  // Only the shelf detail can remove a book from something.
  removable?: boolean;
  dateLabel?: string;
  dateField?: 'added_at' | 'finished_at';
}>(), {
  removable: false,
  dateLabel: '',
  dateField: 'added_at',
});

defineEmits<{
  removeBook: [id: string];
}>();

const contentRef = ref<HTMLElement | null>(null);
const {containerWidth, setupObserver} = useContainerWidth(contentRef);

// The parent renders this component only once it has books, so the element is
// there to measure as soon as it mounts.
onMounted(setupObserver);

const wide = computed(() => containerWidth.value >= MD_BREAKPOINT);
const listCoverWidth = computed(() => (wide.value ? LIST_COVER_LG : LIST_COVER_SM));
const gridTileWidth = computed(() => (wide.value ? GRID_TILE_LG : GRID_TILE_SM));
const spineHeight = computed(() => (wide.value ? SPINE_HEIGHT_LG : SPINE_HEIGHT_SM));
</script>
