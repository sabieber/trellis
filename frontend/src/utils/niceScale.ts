/**
 * Rounds the axis up to a readable maximum and returns the matching tick step,
 * preferring 1/2/5 multiples. Book counts stay on whole numbers.
 */
export function niceScale(max: number, integerOnly: boolean): { max: number; step: number } {
    if (max <= 0) return {max: 1, step: 1};

    const rough = max / 5;
    const magnitude = 10 ** Math.floor(Math.log10(rough));
    const normalized = rough / magnitude;
    const factor = normalized <= 1 ? 1 : normalized <= 2 ? 2 : normalized <= 5 ? 5 : 10;
    let step = factor * magnitude;
    if (integerOnly) step = Math.max(1, Math.round(step));

    return {max: Math.ceil(max / step) * step, step};
}
