use crate::{dtos::requests::RegisterQuery, error::WebauthnError};
use axum::{
    extract::{Extension, Json, Query},
    response::IntoResponse,
};
use tower_sessions::Session;
use tracing::{error, warn};
use tracing_log::log::info;
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
    info!("Registration process starting");
    println!("Aya");

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

    // Add session ID logging
    let session_id = session.id();
    println!("Session ID at initiate: {:?}", session_id);

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

            println!("Successfully saved reg_state to session storage");
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
    session: Session,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    // First, let's check if the session exists and is valid
    let session_id = session.id();
    println!("Current session ID: {:?}", session_id);

    // Get all session data for debugging
    let session_data = session.clone();
    println!("All session data: {:?}", session_data);

    let reg_state_result = session
        .get::<(String, Uuid, PasskeyRegistration)>("reg_state")
        .await;

    println!("Registration state result: {:?}", reg_state_result);

    println!("reg_state_result:: {:#?}", reg_state_result);

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

    println!(
        "Here's what i got from sessoin {} {} {:#?}",
        username, user_unique_id, reg_state
    );

    let _ = session.remove_value("reg_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)
    {
        Ok(sk) => {
            let mut users_guard = app_state.users.lock().await;
            print!("It came here??");

            //TODO: This is where we would store the credential in a db, or persist them in some other way.
            users_guard
                .keys
                .entry(user_unique_id)
                .and_modify(|keys| keys.push(sk.clone()))
                .or_insert_with(|| vec![sk.clone()]);

            users_guard.name_to_id.insert(username, user_unique_id);
            println!("ALl done!");
            Json(serde_json::json!({
                "status": 200,
                "message": "all done"
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
pub async fn initiate_login() {}

/// Verifies the authentication response by comparing if the signature matches the reg_state and also if the signature is being done by a valid public key and completes the login process.
///
/// # Parameters
/// * `ccr` - The signed Client Certification Request
///
/// # Returns
/// Result containing the authentication token upon successful login
pub async fn verify_and_login() {}
