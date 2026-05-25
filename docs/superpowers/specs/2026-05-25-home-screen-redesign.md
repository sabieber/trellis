# Home Screen Redesign

**Date:** 2026-05-25  
**Status:** Approved

## Overview

Overhaul `HomeView.vue` to be a full-width dashboard matching the design language of LibraryView and GoalsView. Shows two actionable sections: current reading goals and currently-reading books with inline progress tracking.

## Backend: New Endpoint

Add `GET /api/readings/active` (POST, auth required) in `readings.rs`.

Query: readings where `user = auth.0 AND finished_at IS NULL AND cancelled_at IS NULL`, inner-joined with books for title, author, google_books_id.

Response per item:
```json
{
  "reading_id": "uuid",
  "book_id": "uuid",
  "title": "string | null",
  "author": "string | null",
  "google_books_id": "string | null",
  "progress": 123,
  "total_pages": 400
}
```

## Layout

Drop `PageContainer`. Outer shell: `min-h-screen bg-base-300 flex flex-col` (same as LibraryView/GoalsView).

Unauthenticated state: centered welcome message (keep existing).

Authenticated state: two sections separated by vertical gap.

## Goals Section

Header row (`px-4`): "Reading Goals" (`font-bold text-lg`) on left, "See all →" RouterLink to `/goals` on right (`text-sm text-primary`).

Goal cards: compact `bg-base-200 rounded-xl p-3` cards showing label, progress bar, and `X/Y` fraction. Capped at 3 — same data/format as current HomeView but new card style. Empty state: "No goals yet." with link to `/goals`.

## Currently Reading Section

Header row: "Currently Reading" + "See all →" RouterLink to `/library`.

Book cards: `bg-base-200 rounded-xl p-3 flex gap-3 items-center`
- Left: `w-12 h-16 rounded-lg` cover thumbnail (Google Books URL) with colored fallback (same `BOOK_COLORS` + hash as LibraryView)
- Middle (flex-1): title (`font-medium text-sm`), author (`text-xs opacity-50`), "Page X of Y · Z%" (`text-xs opacity-50`), progress bar (`h-1.5`)
- Right: "Update" button (`btn btn-sm btn-neutral rounded-full`) → opens `TrackProgressModal` for that reading

After successful update: refresh active readings, show toast.

Empty state: "Nothing in progress yet."

## Files Changed

- `backend/src/readings.rs` — add `list_active_readings` handler and register route
- `frontend/src/views/HomeView.vue` — full rewrite; imports TrackProgressModal, uses local toast
