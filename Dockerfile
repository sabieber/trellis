# syntax=docker/dockerfile:1

# --- Stage 1: build the frontend to static assets ---
FROM node:22-alpine AS frontend
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
# Same-origin API calls: the backend serves these assets, so no host/port.
ENV VITE_API_BASE_URL=""
RUN npm run build-only

# --- Stage 2: build the backend release binary ---
FROM rust:1.88-bookworm AS backend
WORKDIR /code
COPY backend/ ./
RUN cargo build --release

# --- Stage 3: slim runtime image ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /code/target/release/trellis-backend /usr/local/bin/trellis-backend
COPY --from=frontend /frontend/dist ./dist
ENV STATIC_DIR=/app/dist
ENV PORT=5174
EXPOSE 5174
CMD ["trellis-backend"]
