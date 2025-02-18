#![allow(dead_code)]
use std::sync::Arc;

use crate::models::poll::Poll;
use mongodb::Collection;

pub struct PollRepository {
    collection: Collection<Poll>,
}

impl PollRepository {
    pub fn new(db: Arc<mongodb::Database>) -> Self {
        let collection = db.collection::<Poll>("polls");
        Self { collection }
    }
}
