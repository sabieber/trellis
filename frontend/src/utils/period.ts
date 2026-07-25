import moment from 'moment';

/** Human label for a reporting period: the year, or `MMM YYYY` in month mode. */
export function formatPeriod(mode: string, year: number, month: number): string {
    return mode === 'year'
        ? `${year}`
        : moment().year(year).month(month - 1).format('MMM YYYY');
}
