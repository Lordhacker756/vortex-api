use crate::error::JwtError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: i64,    // expiration time
    pub iat: i64,    // issued at
}

impl Claims {
    pub fn new(user_id: String) -> Self {
        let now = Utc::now();
        Claims {
            sub: user_id,
            iat: now.timestamp(),
            exp: (now + Duration::days(7)).timestamp(),
        }
    }
}

pub fn create_token(user_id: &str, secret: &[u8]) -> Result<String, JwtError> {
    let claims = Claims::new(user_id.to_string());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| JwtError::TokenCreationError)
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, JwtError> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            _ => JwtError::InvalidTokenFormat,
        })?;
    Ok(token_data.claims)
}
