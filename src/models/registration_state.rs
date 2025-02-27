use chrono::{DateTime, Utc};
use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationState {
    pub username: String,
    pub user_unique_id: String,
    pub reg_state: PasskeyRegistration,
    #[serde(with = "bson_datetime")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationState {
    pub user_unique_id: String,
    pub auth_state: PasskeyAuthentication,
    #[serde(with = "bson_datetime")]
    pub created_at: DateTime<Utc>,
}

mod bson_datetime {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        BsonDateTime::from_millis(date.timestamp_millis()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        BsonDateTime::deserialize(deserializer).map(|bson_dt| {
            DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_millis(bson_dt.timestamp_millis())
                    .unwrap_or_default(),
                Utc,
            )
        })
    }
}
