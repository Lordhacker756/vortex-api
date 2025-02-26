use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::CorsLayer;

pub fn init_cors() -> CorsLayer {
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
        .allow_origin(["https://votx.vercel.app".parse::<HeaderValue>().unwrap()])
        .expose_headers([axum::http::header::SET_COOKIE]);

    cors
}
