use axum::{Extension, Router};
use config::{
    cors::init_cors, db, logger::initialize_logger, session::init_session, startup::AppState,
};

use dotenvy::dotenv;
use middleware::auth::require_auth;
use routes::poll_route::poll_router;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod config;
mod controllers;
mod dtos;
mod error;
mod middleware;
mod models;
mod repositories;
mod routes;
mod utils;

use crate::routes::auth_route::auth_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    dotenv().ok();

    // Verify JWT_SECRET is set
    if std::env::var("JWT_SECRET").is_err() {
        error!("JWT_SECRET environment variable not set");
        std::process::exit(1);
    }

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
            poll_router().route_layer(axum::middleware::from_fn(require_auth)),
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
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
