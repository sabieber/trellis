import type {Router} from 'vue-router';

// Series pages are keyed by the Open Library series id (e.g. `OL326110L`).
// Central helper so the route name/encoding lives in one place.
export function goToSeries(router: Router, key: string): void {
  router.push({name: 'series-detail', params: {key: encodeURIComponent(key)}});
}
