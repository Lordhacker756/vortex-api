use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::auth_controller::{finish_register, initiate_login, initiate_register};

pub fn auth_router() -> Router {
    Router::new()
        .route("/login", get(initiate_login))
        .route("/register", get(initiate_register))
        .route("/verify-register", post(finish_register))
}
