# Goals View Responsive Redesign

**Date:** 2026-05-25  
**Status:** Approved

## Overview

Overhaul `GoalsView.vue` to fully utilise desktop screens, be fully responsive, and match the visual design language of `LibraryView.vue`. The current view uses `PageContainer` which caps width at `max-w-lg`, making it look narrow on desktop.

## Approach

Goals are grouped by timeframe (Yearly → Monthly → Weekly), mirroring how LibraryView groups books by shelf. Each timeframe section only renders when it has at least one goal.

## Layout & Header

- Drop `PageContainer` entirely.
- Outer shell: `min-h-screen bg-base-300 flex flex-col` — identical to LibraryView.
- Header: `flex justify-between items-center px-4 pt-5 pb-2`
  - Left: `<h1>` "Reading Goals" (`text-2xl font-bold`)
  - Right: `+` button (`btn btn-circle btn-primary`)

## Grouping Structure

Three sections in fixed order: **Yearly**, **Monthly**, **Weekly**. A section is hidden entirely when it has no goals.

Each section header row matches LibraryView's shelf header:
- `flex justify-between items-center mb-3 px-4`
- Left: section name (`font-bold text-lg`) + goal count in `text-sm opacity-50`
- No "See all" link (goals are shown in full, not truncated)

## Goal Cards

Cards inside each section use a responsive grid:
```
grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 px-4
```

Each card: `bg-base-200 rounded-xl p-4`

Card layout:
1. Top row: goal label (`font-semibold text-lg`) + `TrashIcon` delete button (`btn btn-ghost btn-sm btn-square`, top right)
2. Below label: date range (`text-sm opacity-50`)
3. Progress row: `flex justify-between text-sm mb-1` — fraction left (`X / Y books/pages`), percentage right
4. Progress bar: `progress w-full`, `progress-success` when ≥ 100%, `progress-primary` otherwise

## Loading & Empty States

- Loading: `flex justify-center py-10` with `loading-spinner loading-lg` — same as LibraryView
- Empty (no goals at all): centered `opacity-50 py-10` message "No goals yet. Create one to start tracking!"

## Toast

Inline toast in the view template (same pattern as LibraryView), removing reliance on `PageContainer.showToast`. The `pageContainer` ref and its `showToast` calls are replaced with a local `toastMessage` / `toastType` ref pair and a `showToast` helper with a 3-second timeout.

## Files Changed

- `frontend/src/views/GoalsView.vue` — full rewrite of template; minor script updates (add local toast state, remove pageContainer ref)

## Files Unchanged

- `frontend/src/components/CreateGoalModal.vue` — no changes needed
- `frontend/src/components/PageContainer.vue` — no changes needed
- All other views — unaffected
