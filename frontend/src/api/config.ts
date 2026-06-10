// Empty default = same-origin requests (e.g. `/api/...`). In production the
// backend serves the frontend; in dev the vite proxy forwards `/api` to the
// backend. Override with VITE_API_BASE_URL only for unusual setups.
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '';
