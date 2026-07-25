import {reactive} from 'vue';

/**
 * A single floating tooltip shared by all the stats charts. Native SVG/HTML
 * `title` tooltips never fire on touch, so marks bind pointer handlers instead:
 * a mouse hover follows the cursor, a touch/pen tap pins the tooltip until the
 * next tap elsewhere. One `<ChartTooltip>` instance renders `state`.
 */
const state = reactive({visible: false, x: 0, y: 0, content: ''});
let pinned = false;

function show(content: string, x: number, y: number, pin: boolean) {
    state.content = content;
    state.x = x;
    state.y = y;
    state.visible = true;
    pinned = pin;
}

function hide() {
    state.visible = false;
    pinned = false;
}

/**
 * Pointer handlers for one interactive mark, to spread with `v-on`. `content`
 * is resolved lazily so nothing is computed until the mark is actually touched.
 * An empty string suppresses the tooltip (e.g. calendar padding cells).
 */
function marks(content: () => string) {
    return {
        pointerenter: (e: PointerEvent) => {
            if (e.pointerType !== 'mouse') return;
            const text = content();
            if (text) show(text, e.clientX, e.clientY, false);
        },
        pointermove: (e: PointerEvent) => {
            if (e.pointerType === 'mouse' && state.visible) {
                state.x = e.clientX;
                state.y = e.clientY;
            }
        },
        pointerleave: (e: PointerEvent) => {
            if (e.pointerType === 'mouse') hide();
        },
        pointerdown: (e: PointerEvent) => {
            if (e.pointerType === 'mouse') return;
            const text = content();
            if (!text) return;
            show(text, e.clientX, e.clientY, true);
            // Keep the document dismiss handler from firing for this same tap.
            e.stopPropagation();
        },
    };
}

export function useChartTooltip() {
    return {state, hide, marks, isPinned: () => pinned};
}
