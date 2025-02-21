use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use tower_sessions::Session;

pub async fn require_login(
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    println!("Incoming request:: {:#?}", req);
    let session = req
        .extensions()
        .get::<Session>()
        .ok_or((StatusCode::UNAUTHORIZED, "No session"))?;
    match session.get::<String>("user_id").await {
        Ok(_res) => match _res {
            Some(_) => Ok(next.run(req).await),
            None => Err((StatusCode::UNAUTHORIZED, "Not logged in")),
        },
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Not logged in")),
    }
}
