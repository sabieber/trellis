mod books;
mod book_search;
mod db;
mod goals;
mod google_books_client;
mod goodreads_importer;
mod models;
mod open_library_client;
mod readings;
mod schema;
mod shelves;
mod users;
mod auth;

use axum::Router;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use serde::Serialize;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

/// Migrations compiled into the binary from the `migrations/` directory.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Apply any pending database migrations. Panics (non-zero exit) on failure so
/// the app never serves traffic against a broken schema.
fn run_migrations() {
    let mut conn = db::connect();
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run database migrations");
    info!("database migrations applied");
}

/// Build the application router: all `/api/*` routes plus a static file
/// service for the built frontend, with an `index.html` fallback so that
/// client-side (vue-router) deep links resolve to the SPA.
pub fn build_router(static_dir: PathBuf) -> Router {
    let index_path = static_dir.join("index.html");
    let static_service = ServeDir::new(static_dir).fallback(ServeFile::new(index_path));

    let mut router = Router::new();
    router = users::register_routes(router);
    router = shelves::register_routes(router);
    router = books::register_routes(router);
    router = readings::register_routes(router);
    router = goals::register_routes(router);
    router = book_search::register_routes(router);

    // Anything not matched by an `/api/*` route is served from the static dir.
    router.fallback_service(static_service)
}

/// Main entry point for the application and basic setup of the web server.
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv().ok();
    auth::init_jwt_secret();

    run_migrations();

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_string());
    let port: u16 = std::env::var("PORT")
        .map(|p| p.parse::<u16>().expect("PORT must be a valid u16"))
        .unwrap_or(5174);

    info!("initializing router...");

    let router = build_router(PathBuf::from(static_dir));

    info!("starting server...");

    let address = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    info!("server started on port {}", port);

    axum::serve(listener, router).await.unwrap();
}

/// Response type for failed requests.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::build_router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn serves_index_html_for_unknown_spa_route() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "<!doctype html><title>trellis</title>").unwrap();

        let app = build_router(dir.path().to_path_buf());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/library/some-deep-link")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8_lossy(&body).contains("trellis"));
    }

    #[tokio::test]
    async fn serves_static_asset_when_present() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "<!doctype html>").unwrap();
        std::fs::write(dir.path().join("app.js"), "console.log('hi')").unwrap();

        let app = build_router(dir.path().to_path_buf());

        let response = app
            .oneshot(Request::builder().uri("/app.js").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8_lossy(&body).contains("console.log"));
    }
}
