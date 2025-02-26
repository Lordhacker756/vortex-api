use std::sync::Arc;

use mongodb::Database;
use tower_sessions::{cookie::time, Expiry, MemoryStore, SessionManagerLayer};

use crate::{app, config::startup::AppState};

pub async fn setup_test_app() -> (axum::Router, Arc<Database>) {
    let db = mongodb::Client::with_uri_str(
        "mongodb+srv://utkarshmishra2001:GPJhXKhJfkGEIywT@database.m9v44.mongodb.net/tests",
    )
    .await
    .unwrap()
    .database("test_db");

    let app_state = AppState::new();

    // Create session store and layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));

    // Create the app with session layer
    let app = app::create_app(Arc::new(db.clone()), app_state, session_layer);

    (app, Arc::new(db))
}
