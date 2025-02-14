use anyhow::Error;
use std::sync::Arc;

use mongodb::Collection;

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
}
