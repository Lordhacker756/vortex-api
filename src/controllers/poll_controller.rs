use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query},
    http::{self, StatusCode},
    response::{sse::Event, IntoResponse, Response, Sse},
    Extension, Json,
};
use axum_extra::{headers, TypedHeader};

use chrono::Utc;
use mongodb::Database;
use tokio_stream::{Stream, StreamExt};
use tower_sessions::{session, Session, SessionManager};
use tracing_subscriber::filter;

use crate::{
    dtos::{
        requests::{ResultQueryParams, UpdatePollDTO, VoteQueryParam},
        responses::{PollOptionResponseDTO, PollResponseDTO},
    },
    error::{AppError, PollsError},
    models::poll,
    repositories::poll_repository::PollRepository,
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
    Query(query): Query<VoteQueryParam>,
    session: Session,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    let updated_poll = poll_repository
        .cast_vote(poll_id, query.optionId, session)
        .await?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Vote cast successfully"),
        data: Some(updated_poll),
        timestamp: Utc::now(),
        error: None,
    }))
}

pub async fn can_user_vote(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
    session: Session, // Add session parameter
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);

    // Get user_id from session
    let user_id = session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::InvalidSessionState(e))?
        .ok_or(AppError::AuthenticationFailed)?;

    let can_vote = poll_repository.can_vote(user_id, poll_id).await?;

    Ok(Json(ApiResponse {
        status: if can_vote {
            StatusCode::OK
        } else {
            StatusCode::FORBIDDEN
        }
        .as_u16() as i32,
        message: if can_vote { "Can vote" } else { "Cannot vote" }.to_string(),
        data: Some(can_vote),
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

//*GET:: api/polls/poll_id/results
pub async fn get_poll_result(
    Extension(db): Extension<Arc<Database>>,
    Path(poll_id): Path<String>,
    Query(filters): Query<ResultQueryParams>,
) -> Result<Response, AppError> {
    let poll_repository = poll_repository::PollRepository::new(db);
    // There are 3 cases to get the results

    //* Live results -> Stream The Votes of the given poll_id
    if let Some(true) = filters.live {
        Ok(start_sse(poll_repository, poll_id.clone())
            .await
            .into_response())
    } else {
        Ok(get_poll_result_by_id(poll_repository, poll_id)
            .await?
            .into_response())
    }
}

pub async fn start_sse(
    poll_repository: PollRepository,
    poll_id: String,
) -> Sse<impl Stream<Item = Result<Event, AppError>>> {
    // Create a stream that fetches poll results every second
    let stream =
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .then(move |_| {
                let poll_repo = poll_repository.clone();
                let poll_id = poll_id.clone();

                async move {
                    match poll_repo.get_poll_results(poll_id).await {
                        Ok(poll) => {
                            let options_json =
                                serde_json::to_string(&poll.options).unwrap_or_default();

                            Ok(Event::default().data(options_json).event("poll-update"))
                        }
                        Err(_) => Ok(Event::default()
                            .data("Error fetching poll results")
                            .event("error")),
                    }
                }
            })
            .then(|future: Result<Event, AppError>| async move {
                match future {
                    Ok(event) => Ok(event),
                    Err(e) => Err(e),
                }
            });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
pub async fn get_poll_result_by_id(
    poll_repository: PollRepository,
    poll_id: String,
) -> Result<Json<ApiResponse<PollResponseDTO>>, AppError> {
    let poll = poll_repository.get_poll_results(poll_id).await?;

    Ok(Json(ApiResponse {
        status: http::StatusCode::OK.as_u16() as i32,
        message: String::from("Poll results retrieved successfully"),
        data: Some(poll),
        timestamp: Utc::now(),
        error: None,
    }))
}
