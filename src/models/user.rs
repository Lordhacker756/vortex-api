use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Credential {
    pub credential_id: String,
    pub public_key: String,
    pub sign_count: u32,
    pub attestation_format: String,
    pub device_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub credentials: Option<Vec<Credential>>,
}
