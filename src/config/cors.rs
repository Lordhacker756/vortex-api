use axum::http::HeaderName;
use axum::http::Method;
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn init_cors() -> CorsLayer {
    let allowed_origins = [
        "http://localhost:3000",
        "http://localhost:8000",
        "https://votx.vercel.app",
    ];

    CorsLayer::new()
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-csrf-token"),
            HeaderName::from_static("cookie"),
        ])
        .allow_origin(
            allowed_origins
                .iter()
                .map(|origin| origin.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .expose_headers([axum::http::header::SET_COOKIE, axum::http::header::COOKIE])
        .max_age(Duration::from_secs(86400))
}
