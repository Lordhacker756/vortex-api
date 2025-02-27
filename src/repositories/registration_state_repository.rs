use crate::models::registration_state::{AuthenticationState, RegistrationState};
use anyhow::Error;
use tracing::{info, warn};
use mongodb::{bson::doc, Collection, Database};
use std::sync::Arc;

pub struct RegistrationStateRepository {
    reg_collection: Collection<RegistrationState>,
    auth_collection: Collection<AuthenticationState>,
}

impl RegistrationStateRepository {
    pub fn new(db: Arc<Database>) -> Self {
        let reg_collection = db.collection::<RegistrationState>("registration_states");
        let auth_collection = db.collection::<AuthenticationState>("authentication_states");
        Self {
            reg_collection,
            auth_collection,
        }
    }

    pub async fn save_registration_state(&self, state: RegistrationState) -> Result<(), Error> {
        info!(
            "Saving registration state for user: {} with id: {}",
            state.username, state.user_unique_id
        );

        let result = self.reg_collection.insert_one(&state).await?;

        info!("Registration state saved with ID: {}", result.inserted_id);

        Ok(())
    }

    pub async fn get_and_delete_registration_state(
        &self,
        username: &str,
    ) -> Result<Option<RegistrationState>, Error> {
        info!("Fetching registration state for username: {}", username);

        let state = self
            .reg_collection
            .find_one_and_delete(doc! { "username": username })
            .await?;

        match &state {
            Some(s) => info!(
                "Found registration state for user: {} with id: {}",
                s.username, s.user_unique_id
            ),
            None => warn!("No registration state found for username: {}", username),
        }

        Ok(state)
    }

    pub async fn save_authentication_state(&self, state: AuthenticationState) -> Result<(), Error> {
        self.auth_collection.insert_one(state).await?;
        Ok(())
    }

    pub async fn get_and_delete_authentication_state(
        &self,
        user_id: &str,
    ) -> Result<Option<AuthenticationState>, Error> {
        let state = self
            .auth_collection
            .find_one_and_delete(doc! { "user_unique_id": user_id })
            .await?;
        Ok(state)
    }
}
