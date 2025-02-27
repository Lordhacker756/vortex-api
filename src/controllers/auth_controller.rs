use std::sync::Arc;

use crate::{
    dtos::requests::RegisterQuery,
    error::{AppError, JwtError, WebauthnError},
    models::{
        registration_state::{AuthenticationState, RegistrationState},
        user::User,
    },
    repositories::{
        registration_state_repository::RegistrationStateRepository,
        user_repository::{self, UserRepository},
    },
    utils::jwt::create_token,
};
use axum::{
    extract::{Extension, Json, Path, Query},
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
};
use chrono::Utc;
use mongodb::Database;
use tracing::{error, info, warn};
use webauthn_rs::prelude::*;

use crate::config::startup::AppState;

// Type alias for our custom response type
type AuthResponse = (HeaderMap, Json<serde_json::Value>);

// Helper function to create consistent response type
fn create_auth_response(
    headers: HeaderMap,
    body: serde_json::Value,
) -> Result<AuthResponse, AppError> {
    Ok((headers, Json(body)))
}

fn get_jwt_secret() -> Result<Vec<u8>, AppError> {
    std::env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .map_err(|e| {
            error!("Failed to get JWT_SECRET: {}", e);
            AppError::JwtError(JwtError::MissingSecret)
        })
}

/// Creates a CCR (Client Certification Request) and registration state.
/// Stores the registration state and sends the CCR to the user for signature.
///
/// # Returns
/// Result containing the CCR data for user signature
#[axum::debug_handler]
pub async fn initiate_register(
    Extension(app_state): Extension<AppState>,
    Extension(db): Extension<Arc<Database>>,
    Query(query): Query<RegisterQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("Starting register");

    //getting a unique user_id
    let user_unique_id = {
        let user_guard = app_state.users.lock().await;
        user_guard
            .name_to_id
            .get(&query.username)
            .copied()
            .unwrap_or_else(Uuid::new_v4)
    };

    //exclude existing credentials
    let excluded_credentials = {
        let user_guard = app_state.users.lock().await;
        user_guard.keys.get(&user_unique_id).map(|keys| {
            keys.iter()
                .map(|sk| sk.cred_id().clone())
                .collect::<Vec<_>>()
        })
    };

    // Get the CCR and send to the user, while also saving it to session to verify later
    let res = match app_state.webauthn.start_passkey_registration(
        user_unique_id,
        &query.username,
        &query.username,
        excluded_credentials,
    ) {
        Ok((ccr, reg_state)) => {
            // Store registration state in MongoDB
            let reg_state_repo = RegistrationStateRepository::new(db);
            let registration_state = RegistrationState {
                username: query.username.clone(),
                user_unique_id: user_unique_id.to_string(),
                reg_state,
                created_at: Utc::now(),
            };

            println!(
                "Saving user reg state {} {}",
                registration_state.username, registration_state.user_unique_id
            );

            if let Err(e) = reg_state_repo
                .save_registration_state(registration_state)
                .await
            {
                error!("Failed to save registration state: {:?}", e);
                return Err(AppError::DatabaseError(e.to_string()));
            }

            Ok(Json(ccr))
        }
        Err(e) => {
            warn!("Error in registering challenge -> {:#?}", e);
            Err(AppError::Webauthn(WebauthnError::InvalidCredential))
        }
    };

    Ok(res)
}

/// Processes the signed CCR for user registration.Verify if the CCR is matching the reg_state
/// If yes, then we store the public key and some metadata and the registration is completed
///
/// # Parameters
/// * `ccr` - The signed Client Certification Request
///
/// # Returns
/// A JWT in JSON format when successful otherwise an appropriate JSON error
pub async fn finish_register(
    Extension(app_state): Extension<AppState>,
    Extension(db): Extension<Arc<Database>>,
    Path(username): Path<String>,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<AuthResponse, AppError> {
    let reg_state_repo = RegistrationStateRepository::new(db.clone());

    // Instead of getting from app state, first try to get the registration state
    let reg_state = match reg_state_repo
        .get_and_delete_registration_state(&username) // Pass empty string for user_id
        .await
    {
        Ok(Some(state)) => state,
        Ok(None) => {
            error!("No registration state found for username {}", username);
            return Err(AppError::Webauthn(WebauthnError::CorruptSession));
        }
        Err(e) => {
            error!("Database error while fetching registration state: {}", e);
            return Err(AppError::DatabaseError(e.to_string()));
        }
    };

    // Use the user_unique_id from the registration state
    let user_unique_id = Uuid::parse_str(&reg_state.user_unique_id)
        .map_err(|_| AppError::Webauthn(WebauthnError::CorruptSession))?;

    // Verify only the username since we're getting user_id from reg_state
    if reg_state.username != username {
        error!(
            "Registration state username mismatch. Expected: {}, Found: {}",
            username, reg_state.username
        );
        return Err(AppError::Webauthn(WebauthnError::CorruptSession));
    }

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state.reg_state)
    {
        Ok(sk) => {
            let mut users_guard = app_state.users.lock().await;

            //TODO Check if the user is creating a new passkey or it's a new user
            let user_repository = user_repository::UserRepository::new(db);

            match user_repository.get_user_by_username(username.clone()).await {
                Ok(res) => {
                    match res {
                        Some(mut user) => {
                            //? 1. Existing User
                            //append the new Passkey to their credentials vector and save it
                            user.credentials.push(sk.clone());
                            info!("Passkey Added to User:: {:#?}", user);

                            //Save this user
                            match user_repository
                                .update_user(user.credentials, username.clone())
                                .await
                            {
                                Ok(_) => {
                                    info!("New creds added to user!");
                                }
                                Err(e) => {
                                    error!("Error updating user creds {:#?}", e.to_string());
                                }
                            }
                        }
                        None => {
                            //* 2. New User
                            // Create a new User
                            let user = User {
                                user_id: user_unique_id.to_string(),
                                username: username.clone(),
                                credentials: vec![sk.clone()],
                            };

                            if let Err(e) = user_repository.create_user(user).await {
                                error!("Failed to save user to database: {:?}", e);
                                return Err(AppError::Webauthn(WebauthnError::Unknown));
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Database error: {}", e);
                    return Err(AppError::DatabaseError(e.to_string()));
                }
            }

            // Update in-memory state
            users_guard
                .keys
                .entry(user_unique_id)
                .and_modify(|keys| keys.push(sk.clone()))
                .or_insert_with(|| vec![sk.clone()]);

            users_guard.name_to_id.insert(username, user_unique_id);

            // Generate JWT token
            let jwt_secret = get_jwt_secret()?;
            let token = create_token(&user_unique_id.to_string(), &jwt_secret)?;
            let headers = set_auth_cookie(&token);

            create_auth_response(
                headers,
                serde_json::json!({
                    "status": 200,
                    "message": "Registration completed successfully",
                    "user_id": user_unique_id.to_string()
                }),
            )
        }
        Err(e) => {
            error!("challenge_register -> {:?}", e);
            create_auth_response(
                HeaderMap::new(),
                serde_json::json!({
                    "status": 400,
                    "message": e.to_string()
                }),
            )
        }
    };

    res
}

/// Initiates the login process by creating necessary challenge and maintaining a reg_state
///
/// # Returns
/// Result containing the authentication challenge (CCR)
pub async fn initiate_login(
    Extension(db): Extension<Arc<Database>>,
    Extension(app_state): Extension<AppState>,
    Query(query): Query<RegisterQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("Start Authentication");

    let user_repository = UserRepository::new(db.clone());

    // Get the set of keys that the user possesses
    let mut users_guard = app_state.users.lock().await;

    info!("Finding user by username {}", &query.username);
    // Look up their unique id from the username
    let user_unique_id = match users_guard.name_to_id.get(&query.username).copied() {
        Some(id) => id,
        None => {
            info!("User not found in apstate, checking db");
            // User wasn't there in memory, let's check the db for the same
            match user_repository
                .get_user_by_username(query.username.clone())
                .await
            {
                Ok(Some(user)) => {
                    info!("Found user by username with user_id {:#?}", user.user_id);
                    let uuid =
                        Uuid::parse_str(&user.user_id).map_err(|_| WebauthnError::Unknown)?;

                    // Update the in-memory state with the user's credentials
                    users_guard.name_to_id.insert(query.username, uuid);
                    users_guard.keys.insert(uuid, user.credentials);

                    uuid
                }
                Ok(None) => return Err(AppError::UserNotFound),
                Err(e) => return Err(AppError::DatabaseError(e.to_string())),
            }
        }
    };

    let allow_credentials = users_guard
        .keys
        .get(&user_unique_id)
        .ok_or(AppError::Webauthn(WebauthnError::UserHasNoCredentials))?;

    let res = match app_state
        .webauthn
        .start_passkey_authentication(allow_credentials)
    {
        Ok((rcr, auth_state)) => {
            let reg_state_repo = RegistrationStateRepository::new(db);
            let authentication_state = AuthenticationState {
                user_unique_id: user_unique_id.to_string(),
                auth_state,
                created_at: Utc::now(),
            };

            if let Err(e) = reg_state_repo
                .save_authentication_state(authentication_state)
                .await
            {
                error!("Failed to save authentication state: {:?}", e);
                return Err(AppError::DatabaseError(e.to_string()));
            }

            Json(rcr)
        }
        Err(e) => {
            info!("challenge_authenticate -> {:?}", e);
            return Err(AppError::Webauthn(WebauthnError::Unknown));
        }
    };
    Ok(res)
}

/// Verifies the authentication response by comparing if the signature matches the reg_state and also if the signature is being done by a valid public key and completes the login process.
///
/// # Parameters
/// * `ccr` - The signed Client Certification Request
///
/// # Returns
/// Result containing the authentication token upon successful login
pub async fn verify_and_login(
    Extension(app_state): Extension<AppState>,
    Extension(db): Extension<Arc<Database>>,
    Path(username): Path<String>,
    Json(auth): Json<PublicKeyCredential>,
) -> Result<AuthResponse, AppError> {
    let reg_state_repo = RegistrationStateRepository::new(db);

    let user_unique_id = {
        let user_guard = app_state.users.lock().await;
        match user_guard.name_to_id.get(&username) {
            Some(id) => *id,
            None => {
                error!("User not found in app state: {}", username);
                return Err(AppError::UserNotFound);
            }
        }
    };

    let auth_state = match reg_state_repo
        .get_and_delete_authentication_state(&user_unique_id.to_string())
        .await
    {
        Ok(Some(state)) => {
            // Verify the user_id matches
            if state.user_unique_id != user_unique_id.to_string() {
                error!(
                    "Authentication state mismatch for user_id {}",
                    user_unique_id
                );
                return Err(AppError::Webauthn(WebauthnError::CorruptSession));
            }
            state
        }
        Ok(None) => {
            error!(
                "No authentication state found for user_id {}",
                user_unique_id
            );
            return Err(AppError::Webauthn(WebauthnError::CorruptSession));
        }
        Err(e) => {
            error!("Database error while fetching authentication state: {}", e);
            return Err(AppError::DatabaseError(e.to_string()));
        }
    };

    let res = match app_state
        .webauthn
        .finish_passkey_authentication(&auth, &auth_state.auth_state)
    {
        Ok(auth_result) => {
            let mut users_guard = app_state.users.lock().await;

            // Update the credential counter, if possible.
            users_guard
                .keys
                .get_mut(&user_unique_id)
                .map(|keys| {
                    keys.iter_mut().for_each(|sk| {
                        sk.update_credential(&auth_result);
                    })
                })
                .ok_or(AppError::Webauthn(WebauthnError::UserHasNoCredentials))?;

            // Generate JWT token
            let jwt_secret = get_jwt_secret()?;
            let token = create_token(&auth_state.user_unique_id, &jwt_secret)?;
            let headers = set_auth_cookie(&token);

            create_auth_response(
                headers,
                serde_json::json!({
                    "status": 200,
                    "message": "Login successful",
                    "user_id": auth_state.user_unique_id,
                    "timestamp": Utc::now().to_rfc3339()
                }),
            )
        }
        Err(e) => {
            error!("Authentication failed: {:?}", e);
            return Err(AppError::AuthenticationFailed);
        }
    };

    res
}

pub async fn logout() -> Result<AuthResponse, AppError> {
    let headers = clear_auth_cookie();

    create_auth_response(
        headers,
        serde_json::json!({
            "status": 200,
            "message": "Logged out successfully"
        }),
    )
}

fn set_auth_cookie(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let cookie = format!(
        "authToken={}; HttpOnly; Path=/; SameSite=None; Secure; Max-Age=604800",
        token
    );
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers
}

fn clear_auth_cookie() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        "authToken=; HttpOnly; Path=/; SameSite=None; Secure; Max-Age=0"
            .parse()
            .unwrap(),
    );
    headers
}
