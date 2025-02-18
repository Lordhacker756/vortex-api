use std::sync::Arc;

use crate::{
    dtos::requests::RegisterQuery,
    error::WebauthnError,
    models::user::User,
    repositories::user_repository::{self, UserRepository},
};
use axum::{
    extract::{Extension, Json, Query},
    response::IntoResponse,
};
use chrono::Utc;
use mongodb::Database;
use tower_sessions::Session;
use tracing::{error, info, warn};
use webauthn_rs::prelude::*;

use crate::config::startup::AppState;

/// Creates a CCR (Client Certification Request) and registration state.
/// Stores the registration state and sends the CCR to the user for signature.
///
/// # Returns
/// Result containing the CCR data for user signature
#[axum::debug_handler]
pub async fn initiate_register(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Query(query): Query<RegisterQuery>,
) -> Result<impl IntoResponse, WebauthnError> {
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

    //remove any previous registration states
    let _ = session.remove_value("reg_state").await;

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
            // Store the data with explicit typing
            let session_data: (String, Uuid, PasskeyRegistration) =
                (query.username.clone(), user_unique_id, reg_state);

            session
                .insert("reg_state", session_data)
                .await
                .map_err(|e| {
                    error!("Failed to save to session: {:?}", e);
                    WebauthnError::Unknown
                })?;

            Ok(Json(ccr))
        }
        Err(e) => {
            warn!("Error in registering challenge -> {:#?}", e);
            Err(WebauthnError::Unknown)
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
    session: Session,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let reg_state_result = session
        .get::<(String, Uuid, PasskeyRegistration)>("reg_state")
        .await;

    let (username, user_unique_id, reg_state) = match reg_state_result {
        Ok(Some(state_data)) => state_data,
        Ok(None) => {
            error!("No registration state found in session");
            return Err(WebauthnError::CorruptSession);
        }
        Err(e) => {
            error!("Failed to deserialize session data: {:?}", e);
            return Err(WebauthnError::CorruptSession);
        }
    };

    let _ = session.remove_value("reg_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)
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
                                return Err(WebauthnError::Unknown);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error:: {:#?}", e.to_string())
                }
            }

            // Update in-memory state
            users_guard
                .keys
                .entry(user_unique_id)
                .and_modify(|keys| keys.push(sk.clone()))
                .or_insert_with(|| vec![sk.clone()]);

            users_guard.name_to_id.insert(username, user_unique_id);

            info!("User registration completed successfully");
            Json(serde_json::json!({
                "status": 200,
                "message": "Registration completed successfully"
            }))
        }
        Err(e) => {
            error!("challenge_register -> {:?}", e);
            Json(serde_json::json!({
                "status": 400,
                "message": "Bad request"
            }))
        }
    };

    Ok(res)
}

/// Initiates the login process by creating necessary challenge and maintaining a reg_state
///
/// # Returns
/// Result containing the authentication challenge (CCR)
pub async fn initiate_login(
    Extension(db): Extension<Arc<Database>>,
    Extension(app_state): Extension<AppState>,
    session: Session,
    Query(query): Query<RegisterQuery>,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start Authentication");
    // We get the username from the URL, but you could get this via form submission or
    // some other process.

    // Remove any previous authentication that may have occured from the session.
    let _ = session.remove_value("auth_state").await;

    let user_repository = UserRepository::new(db);

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
                Ok(None) => return Err(WebauthnError::UserNotFound),
                Err(_) => return Err(WebauthnError::Unknown),
            }
        }
    };

    let allow_credentials = users_guard
        .keys
        .get(&user_unique_id)
        .ok_or(WebauthnError::UserHasNoCredentials)?;

    let res = match app_state
        .webauthn
        .start_passkey_authentication(allow_credentials)
    {
        Ok((rcr, auth_state)) => {
            // Drop the mutex to allow the mut borrows below to proceed
            drop(users_guard);

            // Note that due to the session store in use being a server side memory store, this is
            // safe to store the auth_state into the session since it is not client controlled and
            // not open to replay attacks. If this was a cookie store, this would be UNSAFE.
            session
                .insert("auth_state", (user_unique_id, auth_state))
                .await
                .expect("Failed to insert");
            Json(rcr)
        }
        Err(e) => {
            info!("challenge_authenticate -> {:?}", e);
            return Err(WebauthnError::Unknown);
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
    session: Session,
    Json(auth): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let (user_unique_id, auth_state): (Uuid, PasskeyAuthentication) = session
        .get("auth_state")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    let _ = session.remove_value("auth_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_authentication(&auth, &auth_state)
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
                .ok_or(WebauthnError::UserHasNoCredentials)?;

            info!("Authentication Successful!");
            Json(serde_json::json!({
                "status": 200,
                "message": "Login successful",
                "user_id": user_unique_id.to_string(),
                "timestamp": Utc::now().to_rfc3339()
            }))
        }
        Err(e) => {
            error!("Authentication failed: {:?}", e);
            Json(serde_json::json!({
                "status": 400,
                "message": "Authentication failed",
                "error": format!("{:?}", e),
                "timestamp": Utc::now().to_rfc3339()
            }))
        }
    };

    Ok(res)
}
