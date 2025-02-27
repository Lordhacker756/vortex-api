use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::auth_controller::{
    finish_register, initiate_login, initiate_register, logout, verify_and_login,
};

pub fn auth_router() -> Router {
    Router::new()
        .route("/login", get(initiate_login))
        .route("/verify-login/{username}", post(verify_and_login))
        .route("/register", get(initiate_register))
        .route("/verify-register/{username}", post(finish_register))
        .route("/logout", post(logout))
}
