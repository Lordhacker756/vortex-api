use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Credential {
    credential_id: String,
    public_key: String,
    sign_count: u32,
    attestation_format: String,
    device_type: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    user_id: String,
    username: String,
    credentials: Vec<Credential>,
}