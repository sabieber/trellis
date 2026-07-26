import moment from 'moment';
import i18n from '@/i18n';

/** Human label for a reporting period: the year, `MMM YYYY` in month mode, or
 *  the localized "all time" label in total mode. */
export function formatPeriod(mode: string, year: number, month: number): string {
    if (mode === 'total') return i18n.global.t('stats.allTime');
    return mode === 'year'
        ? `${year}`
        : moment().year(year).month(month - 1).format('MMM YYYY');
}
