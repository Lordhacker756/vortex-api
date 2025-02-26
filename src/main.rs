use axum::{Extension, Router};
use config::{
    cors::init_cors, db, logger::initialize_logger, session::init_session, startup::AppState,
};
use dotenvy;

use middleware::auth::require_login;
use routes::poll_route::poll_router;
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod controllers;
mod dtos;
mod error;
mod middleware;
mod models;
mod repositories;
mod routes;

use crate::routes::auth_route::auth_router;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize logging
    initialize_logger();

    info!("🚀 Server starting initialization...");

    // Initialize Database
    let db = db::init_database()
        .await
        .expect("Failed to initialize database");

    // Initialize App State
    let app_state = AppState::new();

    let app = Router::new()
        .nest("/api/auth", auth_router())
        .nest(
            "/api/polls",
            poll_router().route_layer(axum::middleware::from_fn(require_login)),
        )
        .layer(init_cors())
        .layer(init_session())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db))
        .layer(Extension(app_state));

    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    info!("🚀 Server started successfully at port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}
