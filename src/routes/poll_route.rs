use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::{
    controllers::poll_controller::{
        can_user_vote, cast_vote, close_poll_by_id, create_new_poll, get_all_polls, get_poll_by_id,
        get_poll_live_results, get_poll_result, manage_all_polls, reset_poll_by_id,
        update_poll_by_id,
    },
    middleware::auth::require_auth,
};

pub fn poll_router() -> Router {
    Router::new()
        .route(
            "/",
            get(get_all_polls).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/",
            post(create_new_poll).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/{poll_id}",
            get(get_poll_by_id).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/{poll_id}",
            patch(update_poll_by_id).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/{poll_id}/vote",
            get(cast_vote).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/{poll_id}/can-vote",
            get(can_user_vote).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/{poll_id}/reset",
            get(reset_poll_by_id).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/{poll_id}/close",
            get(close_poll_by_id).route_layer(axum::middleware::from_fn(require_auth)),
        )
        .route(
            "/manage",
            get(manage_all_polls).route_layer(axum::middleware::from_fn(require_auth)),
        )
        // public routes
        .route("/{poll_id}/results", get(get_poll_result))
        .route("/{poll_id}/results/live", get(get_poll_live_results))
}
