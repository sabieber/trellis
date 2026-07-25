import {ref, watch, type Ref} from 'vue';
import {apiFetch} from '@/api/client';

/**
 * Loads a period-scoped stats resource and keeps it in sync with the period
 * refs. All stats endpoints share the same `{mode, year, month}` request and
 * the same load/error/loading lifecycle, so the sections build on this instead
 * of repeating it.
 */
export function usePeriodResource<T>(
    path: string,
    mode: Ref<string>,
    year: Ref<number>,
    month: Ref<number>,
) {
    const data = ref<T | null>(null) as Ref<T | null>;
    const loading = ref(true);

    const load = async () => {
        loading.value = true;
        try {
            const res = await apiFetch(path, {
                method: 'POST',
                body: JSON.stringify({
                    mode: mode.value,
                    year: year.value,
                    month: mode.value === 'month' ? month.value : undefined,
                }),
            });
            data.value = res.ok ? await res.json() : null;
        } catch (e) {
            console.error(`Failed to fetch ${path}:`, e);
            data.value = null;
        } finally {
            loading.value = false;
        }
    };

    watch(() => [mode.value, year.value, month.value], load, {immediate: true});

    return {data, loading, reload: load};
}
