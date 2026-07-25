/** Local-time `YYYY-MM-DD` key, matching the `read_at` dates from the API. */
export function dateKey(date: Date): string {
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${date.getFullYear()}-${month}-${day}`;
}

/** Weekday index of a date with Monday as the first day of the week (0-6). */
export function mondayIndex(date: Date): number {
    return (date.getDay() + 6) % 7;
}

/** Number of days in the given month (1-12). */
export function daysInMonth(year: number, month: number): number {
    return new Date(year, month, 0).getDate();
}

/**
 * Quartile boundaries of the non-zero values, used to map a day's volume onto
 * the four heat levels of the activity grid. Falls back to even steps towards
 * the maximum when the values are too uniform for quartiles to differ.
 */
export function heatThresholds(values: number[]): [number, number, number] {
    const sorted = values.filter((value) => value > 0).sort((a, b) => a - b);
    if (sorted.length === 0) return [1, 2, 3];

    const quantile = (p: number) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))];
    const max = sorted[sorted.length - 1];
    const boundaries: [number, number, number] = [quantile(0.25), quantile(0.5), quantile(0.75)];

    if (boundaries[0] >= boundaries[2]) {
        return [max * 0.25, max * 0.5, max * 0.75];
    }
    return boundaries;
}

/** Maps a day's volume onto a heat level, `0` meaning no activity at all. */
export function heatLevel(value: number, thresholds: [number, number, number]): number {
    if (value <= 0) return 0;
    if (value < thresholds[0]) return 1;
    if (value < thresholds[1]) return 2;
    if (value < thresholds[2]) return 3;
    return 4;
}
