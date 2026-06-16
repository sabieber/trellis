import {ref, onUnmounted, nextTick, watch, type Ref} from 'vue';

export function useContainerWidth(elRef: Ref<HTMLElement | null>, readyRef?: Ref<boolean>) {
    const containerWidth = ref(0);
    let observer: ResizeObserver | null = null;

    const setup = () => {
        if (elRef.value && !observer) {
            containerWidth.value = elRef.value.clientWidth;
            observer = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    containerWidth.value = entry.contentRect.width;
                }
            });
            observer.observe(elRef.value);
        }
    };

    if (readyRef) {
        watch(readyRef, (val) => {
            if (val) nextTick(setup);
        });
    }

    onUnmounted(() => {
        observer?.disconnect();
    });

    return {containerWidth, setupObserver: setup};
}
