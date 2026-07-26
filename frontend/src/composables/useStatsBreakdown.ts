import {type Ref} from 'vue';
import {usePeriodResource} from './usePeriodResource';

/** An author and how much of them was read in the period. */
export interface AuthorStat {
    author: string;
    /** Finished readings of this author (re-reads count separately). */
    books: number;
    /** Summed pages of those finished readings. */
    pages: number;
}

/** A finished book highlighted in the period's book lists. */
export interface BookStat {
    book_id: string;
    title: string;
    author: string;
    cover_url: string | null;
    /** Star rating of the book (present in `top_rated`). */
    rating?: number;
    /** Readings of this book finished in the period (present in `most_read`). */
    readings?: number;
}

/** Readings by outcome within the period. The buckets are disjoint. */
export interface ReadingStates {
    finished: number;
    reading: number;
    abandoned: number;
}

/** Aggregate breakdowns of a reporting period. */
export interface StatsBreakdown {
    mode: string;
    year: number;
    month: number | null;
    /** Counts of finished books per star rating, index 0 (one star) to 4 (five stars). */
    rating_distribution: number[];
    /** Counts of finished books per 100-page band, index 0 (0–99) upward. */
    page_distribution: number[];
    top_authors: AuthorStat[];
    /** Up to three finished books with the highest rating, best first. */
    top_rated: BookStat[];
    /** Up to three books with the most finished readings, most read first. */
    most_read: BookStat[];
    reading_states: ReadingStates;
}

/**
 * Loads the aggregate breakdowns (ratings, authors, states, weekdays) of the
 * given period. Fetched once and shared across the breakdown sections, in the
 * same spirit as [`useActivityStats`].
 */
export function useStatsBreakdown(mode: Ref<string>, year: Ref<number>, month: Ref<number>) {
    const {data: breakdown, loading, reload} = usePeriodResource<StatsBreakdown>(
        '/api/stats/breakdown',
        mode,
        year,
        month,
    );

    return {breakdown, loading, reload};
}
