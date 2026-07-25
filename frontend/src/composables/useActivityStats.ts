import {computed, type Ref} from 'vue';
import {usePeriodResource} from './usePeriodResource';

/** A single day with logged reading activity. */
export interface ActivityDay {
    /** The day in `YYYY-MM-DD` format. */
    date: string;
    /** Pages logged on that day. */
    pages: number;
    /** Readings finished on that day. */
    books: number;
}

/** The reading activity of a reporting period. Days without activity are omitted. */
export interface ActivitySeries {
    mode: string;
    year: number;
    month: number | null;
    start: string;
    end: string;
    days: ActivityDay[];
}

/**
 * Loads the day-by-day reading activity of the given period and keeps it in
 * sync with the period refs. Both the activity grid and the bar chart build on
 * this, so the series is fetched once and shared instead of per section.
 */
export function useActivityStats(mode: Ref<string>, year: Ref<number>, month: Ref<number>) {
    const {data: series, loading, reload} = usePeriodResource<ActivitySeries>(
        '/api/stats/activity',
        mode,
        year,
        month,
    );

    /** Activity by `YYYY-MM-DD`, for cheap day lookups while rendering. */
    const byDate = computed(() => {
        const map = new Map<string, ActivityDay>();
        for (const day of series.value?.days ?? []) {
            map.set(day.date, day);
        }
        return map;
    });

    return {series, byDate, loading, reload};
}
