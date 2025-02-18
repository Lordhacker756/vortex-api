use axum::{Extension, Router};
use config::{
    cors::init_cors, db, logger::initialize_logger, session::init_session, startup::AppState,
};
use dotenvy;

use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod controllers;
mod dtos;
mod error;
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
        .nest("/auth", auth_router())
        .layer(init_session())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db))
        .layer(Extension(app_state))
        .layer(init_cors());

    info!("🚀 Server started successfully at port 9000");
    let listener = tokio::net::TcpListener::bind("localhost:9000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}
