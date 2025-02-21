use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query},
    http::{self, StatusCode},
    response::{sse::Event, Sse},
    Extension, Json,
};
use axum_extra::{headers, TypedHeader};

use chrono::Utc;
use mongodb::Database;
use tokio_stream::{Stream, StreamExt};
use tower_sessions::{session, Session, SessionManager};

use crate::{
    dtos::{
        requests::UpdatePollDTO,
        responses::{PollOptionResponseDTO, PollResponseDTO},
    },
    error::{AppError, PollsError},
    models::poll,
};

use crate::{
    dtos::{requests::CreatePollDTO, responses::ApiResponse},
    repositories::poll_repository,
};

//*GET:: api/polls
pub async fn get_all_polls(
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<ApiResponse<Vec<PollResponseDTO>>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    match poll_repository.get_all_polls().await {
        Ok(polls) => Ok(Json(ApiResponse {
            status: StatusCode::OK.as_u16() as i32,
            message: String::from("All posts fetched successfully"),
            data: Some(polls),
            timestamp: Utc::now(),
            error: None,
        })),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}
//*GET:: api/polls/manage
pub async fn manage_all_polls(
    Extension(db): Extension<Arc<Database>>,
    session: Session,
) -> Result<Json<ApiResponse<Vec<PollResponseDTO>>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let user_id = session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::SessionExpired)?
        .ok_or(AppError::AuthenticationFailed)?;

    let polls = poll_repository
        .get_polls_of_user(user_id)
        .await?
        .ok_or(AppError::Poll(PollsError::NoPollsFoundForUser))?;

    Ok(Json(ApiResponse {
        status: StatusCode::OK.as_u16() as i32,
        message: String::from("User polls fetched successfully"),
        data: Some(polls),
        timestamp: Utc::now(),
        error: None,
    }))
}

//?POST:: api/polls
pub async fn create_new_poll(
    Extension(db): Extension<Arc<Database>>,
    axum::Json(payload): axum::Json<CreatePollDTO>,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    match poll_repository.create_poll(payload).await {
        Ok(poll) => Ok(Json(ApiResponse {
            status: http::StatusCode::CREATED.as_u16() as i32,
            message: String::from("Poll created successfully"),
            data: Some(poll),
            timestamp: Utc::now(),
            error: None,
        })),
        Err(e) => Err(e),
    }
}

//*GET:: api/polls/poll_id
pub async fn get_poll_by_id(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let poll = poll_repository
        .get_poll_by_id(poll_id)
        .await?
        .ok_or(AppError::Poll(PollsError::PollNotFound))?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Poll retrieved successfully"),
        data: Some(poll),
        timestamp: Utc::now(),
        error: None,
    }))
}

//?PATCH:: api/polls/poll_id
pub async fn update_poll_by_id(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
    axum::Json(payload): axum::Json<UpdatePollDTO>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    match poll_repository.update_poll(poll_id, payload).await {
        Ok(_) => Ok(Json(ApiResponse {
            status: StatusCode::CREATED.as_u16() as i32,
            message: String::from("Poll updated successfully"),
            data: None,
            timestamp: Utc::now(),
            error: None,
        })),
        Err(e) => Err(e),
    }
}

//*GET:: api/polls/poll_id/vote
pub async fn cast_vote(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
    Query(option_id): Query<String>,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let updated_poll = poll_repository.cast_vote(poll_id, option_id).await?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Vote cast successfully"),
        data: Some(updated_poll),
        timestamp: Utc::now(),
        error: None,
    }))
}

//*GET:: api/polls/poll_id/results
pub async fn get_poll_result(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let poll = poll_repository.get_poll_results(poll_id).await?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Poll results retrieved successfully"),
        data: Some(poll),
        timestamp: Utc::now(),
        error: None,
    }))
}

//*GET:: api/polls/poll_id/close
pub async fn close_poll_by_id(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let updated_poll = poll_repository.close_poll(poll_id).await?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Poll closed successfully"),
        data: Some(updated_poll),
        timestamp: Utc::now(),
        error: None,
    }))
}

//*GET:: api/polls/poll_id/reset
pub async fn reset_poll_by_id(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let updated_poll = poll_repository.reset_poll(poll_id).await?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Poll reset successfully"),
        data: Some(updated_poll),
        timestamp: Utc::now(),
        error: None,
    }))
}

#[axum::debug_handler]
pub async fn start_sse(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // Create a stream that emits a new event every second
    let stream =
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(|_| Ok(Event::default().data("ping").id("id").event("message")));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
