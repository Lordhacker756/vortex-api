use crate::utils::jwt::verify_token;
use axum::{
    extract::Request,
    http::{
        header::{AUTHORIZATION, COOKIE},
        StatusCode,
    },
    middleware::Next,
    response::Response,
};

pub async fn require_auth(
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    // Try to get token from cookie first
    let token = req
        .headers()
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|cookie| cookie.trim().starts_with("authToken="))
                .map(|auth_cookie| auth_cookie.trim().strip_prefix("authToken=").unwrap())
        });

    // Fallback to Bearer token if cookie not found
    let token = match token {
        Some(t) => t.to_string(),
        None => {
            // Try Bearer token
            let auth_header = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|header| header.to_str().ok())
                .and_then(|header| header.strip_prefix("Bearer "));

            match auth_header {
                Some(token) => token.to_string(),
                None => return Err((StatusCode::UNAUTHORIZED, "Missing authorization")),
            }
        }
    };

    match verify_token(&token, &std::env::var("JWT_SECRET").unwrap().into_bytes()) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid token")),
    }
}
