use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};

use crate::controllers::poll_controller::{
    cast_vote, close_poll_by_id, create_new_poll, get_all_polls, get_poll_by_id, get_poll_result,
    manage_all_polls, reset_poll_by_id, update_poll_by_id,
};

pub fn poll_router() -> Router {
    Router::new()
        .route("/", get(get_all_polls))
        .route("/", post(create_new_poll))
        .route("/{poll_id}", get(get_poll_by_id))
        .route("/{poll_id}", patch(update_poll_by_id))
        .route("/{poll_id}/vote", get(cast_vote))
        .route("/{poll_id}/results", get(get_poll_result))
        .route("/{poll_id}/reset", get(reset_poll_by_id))
        .route("/{poll_id}/close", get(close_poll_by_id))
        .route("/manage", get(manage_all_polls))
}
