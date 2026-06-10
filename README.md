<p align="center">
    <img src="frontend/public/logo.svg" alt="trellis" width="256" />
</p>

# trellis

Your open source reading tracker.

> Work in progress.

Track books and reading progress, set reading goals, and organise your library — built as a self-hostable web app.

## Tech stack

- **Backend** — Rust
- **Frontend** — Vue 3, TypeScript, Vite, Tailwind CSS / DaisyUI

## Features

- Book search and detail views
- Reading progress tracking
- Personal library with shelves
- Reading goals

## Getting started

First, create your environment file from the template and fill in the values:

```sh
cp .env.example .env
```

Then run the full stack (app + database) with Docker:

```sh
docker compose up --build
```

The app is served at http://localhost:5174. The database schema is created
automatically on first start.

### Local development

For hot-reloading dev servers, run the database in Docker and the backend and
frontend locally. This needs three terminals:

```sh
# terminal 1 — database (Postgres on localhost:5432)
docker compose up db

# terminal 2 — backend (applies migrations, serves the API on :5174)
cd backend && cargo run

# terminal 3 — frontend dev server (proxies /api to the backend)
cd frontend && npm install && npm run dev
```

The app is then served at http://localhost:5173. The backend requires a
reachable database — it applies migrations on startup and exits if it cannot
connect, so start the database (terminal 1) before the backend.
