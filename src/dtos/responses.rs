use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub status: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationSuccessData {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationSuccessData {
    pub user_id: String,
    pub username: String,
    pub is_new_user: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiateRegistrationData {
    pub public_key: CreationChallengeResponse,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiateAuthenticationData {
    pub public_key: RequestChallengeResponse,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResponseDTO {
    pub poll_id: String,
    pub created_by: String,
    pub name: String,
    pub is_multi: bool,
    pub is_paused: bool,
    pub is_closed: bool,
    pub start_date: String,
    pub end_date: String,
    pub options: Vec<PollOptionResponseDTO>,
    pub voted_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollOptionResponseDTO {
    pub option_id: String,
    pub option_name: String,
    pub votes: i32,
}
