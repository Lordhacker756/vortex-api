use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::Passkey;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub credentials: Vec<Passkey>,
}
