use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use config::db;
use dotenvy;

use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    session_store, Expiry, MemoryStore, SessionManagerLayer,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod controllers;
mod dtos;
mod error;
mod models;
mod repositories;
mod routes;

use crate::routes::auth_route::auth_router;
use config::startup::AppState;

async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "RESTful API in Rust using Axum Framework and MongoDB";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO) // Change to TRACE level
        .with_level(true)
        .with_line_number(true)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up logging");

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
            axum::http::header::SET_COOKIE,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true)
        .allow_origin(["http://localhost:8000".parse::<HeaderValue>().unwrap()])
        .expose_headers([axum::http::header::SET_COOKIE]);

    info!("connecting to database");
    let db = db::init_database()
        .await
        .expect("Failed to initialize database");

    let app_state = AppState::new();
    let session_store = MemoryStore::default();

    let app = Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .nest("/auth", auth_router())
        .layer(
            SessionManagerLayer::new(session_store)
                .with_name("webauthnrs")
                .with_same_site(SameSite::Lax)
                .with_secure(false)
                .with_path("/")
                .with_expiry(Expiry::OnInactivity(Duration::seconds(3600))),
        )
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db))
        .layer(Extension(app_state))
        .layer(cors);

    info!("🚀 Server started successfully at port 9000");
    let listener = tokio::net::TcpListener::bind("localhost:9000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}
