use axum::{response::IntoResponse, routing::get, Extension, Json, Router};
use config::db;
use dotenvy;
use models::user::User;
use repositories::{poll, user};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

mod config;
mod models;
mod repositories;

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

    info!("connecting to database");
    let db = db::init_database()
        .await
        .expect("Failed to initialize database");
    info!("successfully connected to database 🍀");

    info!("Setting up user repository ㊫");
    let user_repository = user::UserRepository::new(db.clone());
    info!("Setting up poll repository 🗳️");
    let poll_repository = poll::PollRepository::new(db.clone());

    let app = Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db));

    info!("🚀 Server started successfully at port 9000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}
