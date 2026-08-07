import {ref, onUnmounted, type Ref} from 'vue';

export function useContainerWidth(elRef: Ref<HTMLElement | null>) {
    const containerWidth = ref(0);
    const containerHeight = ref(0);
    let observer: ResizeObserver | null = null;

    const setup = () => {
        if (elRef.value && !observer) {
            containerWidth.value = elRef.value.clientWidth;
            containerHeight.value = elRef.value.clientHeight;
            observer = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    containerWidth.value = entry.contentRect.width;
                    containerHeight.value = entry.contentRect.height;
                }
            });
            observer.observe(elRef.value);
        }
    };

    onUnmounted(() => {
        observer?.disconnect();
    });

    return {containerWidth, containerHeight, setupObserver: setup};
}
