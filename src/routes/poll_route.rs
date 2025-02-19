use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::poll_controller::{
    close_poll_by_id, create_new_poll, get_all_polls, get_poll_by_id, reset_poll_by_id, start_sse,
};

pub fn poll_router() -> Router {
    Router::new()
        .route("/sse", get(start_sse))
        .route("/{poll_id}", get(get_poll_by_id))
        .route("/{poll_id}/results", get(get_poll_by_id))
        .route("/{poll_id}/vote", get(get_poll_by_id))
        .route("/{poll_id}/close", get(close_poll_by_id))
        .route("/{poll_id}/reset", get(reset_poll_by_id))
        .route("/", post(create_new_poll))
        .route("/", get(get_all_polls))
        .route("/manage", get(get_all_polls))
}
