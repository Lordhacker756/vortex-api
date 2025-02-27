use axum::http::HeaderName;
use axum::http::Method;
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn init_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-csrf-token"),
            HeaderName::from_static("cookie"),
        ])
        .allow_origin([
            "http://localhost:8000".parse().unwrap(),
            "https://votx.vercel.app".parse().unwrap(),
        ])
        .expose_headers([axum::http::header::SET_COOKIE, axum::http::header::COOKIE])
        .max_age(Duration::from_secs(86400))
}
