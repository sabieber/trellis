import type {Router} from 'vue-router';

// Series pages are keyed by the Open Library series id (e.g. `OL326110L`).
// Central helper so the route name lives in one place. Vue Router encodes the
// param itself, so do not encode it here.
export function seriesRoute(key: string) {
  return {name: 'series-detail', params: {key}};
}

export function goToSeries(router: Router, key: string): void {
  router.push(seriesRoute(key));
}
