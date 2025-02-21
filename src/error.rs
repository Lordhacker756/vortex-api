#![allow(dead_code)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Something's reeeeealy wrong, send hlp pls")]
    Unknown,

    // Authentication & Session Errors
    #[error("Session expired")]
    SessionExpired,
    #[error("Invalid session state: {0}")]
    InvalidSessionState(#[from] tower_sessions::session::Error),
    #[error("Authentication failed")]
    AuthenticationFailed,

    // User Related Errors
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid username format")]
    InvalidUsername,
    #[error("Username already taken")]
    UsernameTaken,

    // Database Errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    // Rate Limiting
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    // Webauthn Specific Errors
    #[error("Webauthn error: {0}")]
    Webauthn(#[from] WebauthnError),

    // Poll Errors
    #[error("Poll error: {0}")]
    Poll(#[from] PollsError),
}

#[derive(Error, Debug)]
pub enum PollsError {
    #[error("The user hasn't created any polls")]
    NoPollsFoundForUser,

    #[error("Poll not found")]
    PollNotFound,

    #[error("Poll has already ended")]
    PollEnded,

    #[error("Poll is currently paused")]
    PollPaused,

    #[error("Poll has been closed")]
    PollClosed,

    #[error("Invalid poll option")]
    InvalidPollOption,

    #[error("User has already voted on this poll")]
    AlreadyVoted,

    #[error("Invalid poll dates: {0}")]
    InvalidPollDates(String),

    #[error("Poll creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid poll configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Unauthorized to manage this poll")]
    UnauthorizedAccess,

    #[error("Poll update failed: {0}")]
    UpdateFailed(String),

    #[error("Cannot modify closed poll")]
    CannotModifyClosed,
}

#[derive(Error, Debug)]
pub enum WebauthnError {
    #[error("Something's reeeeealy wrong in webauth, send hlp pls")]
    Unknown,
    #[error("Corrupt webauthn session")]
    CorruptSession,
    #[error("Invalid credential format")]
    InvalidCredential,
    #[error("User has no credentials")]
    UserHasNoCredentials,
    #[error("Challenge verification failed")]
    ChallengeVerificationFailed,
    #[error("Invalid attestation")]
    InvalidAttestation,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_string = self.to_string();
        let (status, error_message) = match self {
            // Authentication & Session Errors
            AppError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session Expired"),
            AppError::InvalidSessionState(_) => (StatusCode::BAD_REQUEST, "Invalid Session State"),
            AppError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "Authentication Failed"),

            // User Related Errors
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User Not Found"),
            AppError::InvalidUsername => (StatusCode::BAD_REQUEST, "Invalid Username Format"),
            AppError::UsernameTaken => (StatusCode::CONFLICT, "Username Already Taken"),

            // Database Errors
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),

            // Rate Limiting
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate Limit Exceeded"),

            // Webauthn Errors
            AppError::Webauthn(webauthn_err) => match webauthn_err {
                WebauthnError::Unknown => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Some unknown error occured, we've put our wizards to work!",
                ),
                WebauthnError::CorruptSession => {
                    (StatusCode::BAD_REQUEST, "Corrupt Webauthn Session")
                }
                WebauthnError::InvalidCredential => {
                    (StatusCode::BAD_REQUEST, "Invalid Credential Format")
                }
                WebauthnError::UserHasNoCredentials => {
                    (StatusCode::BAD_REQUEST, "No Credentials Found")
                }
                WebauthnError::ChallengeVerificationFailed => {
                    (StatusCode::BAD_REQUEST, "Challenge Verification Failed")
                }
                WebauthnError::InvalidAttestation => {
                    (StatusCode::BAD_REQUEST, "Invalid Attestation")
                }
            },

            // Poll Errors
            AppError::Poll(poll_err) => match poll_err {
                PollsError::NoPollsFoundForUser => (
                    StatusCode::NOT_FOUND,
                    "No polls found have been created by the given user",
                ),
                PollsError::PollNotFound => (StatusCode::NOT_FOUND, "Poll Not Found"),
                PollsError::PollEnded => (StatusCode::FORBIDDEN, "Poll Has Already Ended"),
                PollsError::PollPaused => (StatusCode::FORBIDDEN, "Poll Is Currently Paused"),
                PollsError::PollClosed => (StatusCode::FORBIDDEN, "Poll Has Been Closed"),
                PollsError::InvalidPollOption => (StatusCode::BAD_REQUEST, "Invalid Poll Option"),
                PollsError::AlreadyVoted => (StatusCode::CONFLICT, "Already Voted On This Poll"),
                PollsError::InvalidPollDates(_) => (StatusCode::BAD_REQUEST, "Invalid Poll Dates"),
                PollsError::CreationFailed(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed To Create Poll")
                }
                PollsError::InvalidConfiguration(_) => {
                    (StatusCode::BAD_REQUEST, "Invalid Poll Configuration")
                }
                PollsError::UnauthorizedAccess => {
                    (StatusCode::FORBIDDEN, "Unauthorized To Manage Poll")
                }
                PollsError::UpdateFailed(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed To Update Poll")
                }
                PollsError::CannotModifyClosed => {
                    (StatusCode::FORBIDDEN, "Cannot Modify Closed Poll")
                }
            },

            AppError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown Error"),
        };

        let body = Json(json!({
            "status": status.as_u16(),
            "message": error_message,
            "error": error_string,
            "timestamp": chrono::Utc::now()
        }));

        (status, body).into_response()
    }
}
