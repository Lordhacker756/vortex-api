use anyhow::Error;
use std::sync::Arc;
use webauthn_rs::prelude::Passkey;

use mongodb::{
    bson::{self, doc},
    Collection,
};

use crate::models::user::User;

pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn new(db: Arc<mongodb::Database>) -> Self {
        let collection = db.collection::<User>("users");
        Self { collection }
    }

    pub async fn create_user(&self, user: User) -> Result<(), Error> {
        let _ = self.collection.insert_one(user).await;
        Ok(())
    }

    pub async fn get_user_by_username(&self, username: String) -> Result<Option<User>, Error> {
        let user: Option<User> = self
            .collection
            .find_one(doc! {
                "username": username
            })
            .await?;
        Ok(user)
    }

    pub async fn update_user(
        &self,
        updated_credentials: Vec<Passkey>,
        username: String,
    ) -> Result<(), Error> {
        let credentials_bson = bson::to_bson(&updated_credentials)?;
        let result = self
            .collection
            .find_one_and_update(
                doc! { "username": username },
                doc! {
                    "$set": {
                        "credentials": credentials_bson
                    }
                },
            )
            .await?;

        match result {
            Some(_) => Ok(()),
            None => Err(anyhow::anyhow!("User not found")),
        }
    }
}
