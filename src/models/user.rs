use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::COSEKeyType;

#[derive(Serialize, Deserialize, Debug)]
pub struct Credential {
    pub credential_id: String,
    pub public_key: COSEKeyType,
    pub sign_count: u32,
    pub device_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub credentials: Option<Vec<Credential>>,
}
