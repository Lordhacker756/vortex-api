#![allow(dead_code)]
use std::sync::Arc;

use crate::{
    dtos::{
        requests::{CreatePollDTO, UpdatePollDTO},
        responses::PollResponseDTO,
    },
    error::{AppError, PollsError},
    models::poll::{Poll, PollOption},
};
use futures::TryStreamExt;
use mongodb::{bson::DateTime as BsonDateTime, Collection};
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct PollRepository {
    polls: Collection<Poll>,
}

impl PollRepository {
    pub fn new(db: Arc<mongodb::Database>) -> Self {
        let polls = db.collection::<Poll>("polls");
        Self { polls }
    }

    pub async fn update_poll(&self, poll_id: String, poll: UpdatePollDTO) -> Result<(), AppError> {
        // First check if poll exists and is not closed
        let _existing_poll = self
            .get_poll_by_id(poll_id.clone())
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))?;

        let update_result = self
            .polls
            .update_one(
                mongodb::bson::doc! { "pollId": poll_id },
                mongodb::bson::doc! {
                    "$set": {
                    "name": poll.name,
                    "isMulti": poll.isMulti,
                    "startDate": BsonDateTime::from_millis(poll.startDate.timestamp_millis()),
                    "endDate": BsonDateTime::from_millis(poll.endDate.timestamp_millis())
                    }
                },
            )
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if update_result.modified_count == 0 {
            return Err(AppError::Poll(PollsError::PollNotFound));
        }

        Ok(())
    }

    pub async fn get_polls_of_user(
        &self,
        user_id: String,
    ) -> Result<Option<Vec<PollResponseDTO>>, AppError> {
        match self
            .polls
            .find(mongodb::bson::doc! {
                "createdBy": user_id
            })
            .await
        {
            Ok(res) => {
                let polls = res
                    .try_collect::<Vec<Poll>>()
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?
                    .into_iter()
                    .map(|poll| poll.to_response_dto())
                    .collect();

                Ok(Some(polls))
            }
            Err(e) => Err(AppError::DatabaseError(e.to_string())),
        }
    }

    pub async fn create_poll(&self, dto: CreatePollDTO) -> Result<PollResponseDTO, AppError> {
        let poll_id = Uuid::new_v4().to_string();

        let poll_options = dto
            .options
            .iter()
            .map(|poll| {
                return PollOption {
                    optionName: poll.clone(),
                    optionId: Uuid::new_v4().to_string(),
                    votes: 0,
                };
            })
            .collect::<Vec<PollOption>>();

        let new_poll = Poll {
            pollId: poll_id,
            createdBy: dto.createdBy,
            startDate: BsonDateTime::from_millis(dto.startDate.timestamp_millis()),
            endDate: BsonDateTime::from_millis(dto.endDate.timestamp_millis()),
            name: dto.name,
            isMulti: dto.isMulti,
            isPaused: false,
            isClosed: false,
            options: poll_options,
            votedBy: [].to_vec(),
        };

        info!("Inserting new poll to db {:#?}", new_poll.pollId);

        match self.polls.insert_one(&new_poll).await {
            Ok(_) => Ok(new_poll.to_response_dto()),
            Err(e) => Err(AppError::DatabaseError(e.to_string())),
        }
    }

    pub async fn get_all_polls(&self) -> Result<Vec<PollResponseDTO>, AppError> {
        let polls = self
            .polls
            .find(mongodb::bson::doc! {})
            .sort(mongodb::bson::doc! { "startDate": -1 }) // -1 for descending order
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let poll_list: Vec<PollResponseDTO> = polls
            .try_collect::<Vec<Poll>>()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(|poll| poll.to_response_dto())
            .collect();

        Ok(poll_list)
    }

    pub async fn get_poll_by_id(
        &self,
        poll_id: String,
    ) -> Result<Option<PollResponseDTO>, AppError> {
        let poll = self
            .polls
            .find_one(mongodb::bson::doc! { "pollId": poll_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(poll.map(|p| p.to_response_dto()))
    }

    pub async fn can_vote(&self, user_id: String, poll_id: String) -> Result<bool, AppError> {
        match self.get_poll_by_id(poll_id).await {
            Ok(Some(poll)) => {
                // Check if user has already voted
                let has_voted = poll.voted_by.contains(&user_id);
                Ok(!has_voted)
            }
            Ok(None) => Err(AppError::Poll(PollsError::PollNotFound)),
            Err(e) => Err(AppError::DatabaseError(e.to_string())),
        }
    }

    pub async fn cast_vote(
        &self,
        poll_id: String,
        option_id: String,
        session: Session,
    ) -> Result<PollResponseDTO, AppError> {
        let poll = self
            .get_poll_by_id(poll_id.clone())
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))?;

        let user_id: String = session
            .get("user_id")
            .await
            .map_err(|e| AppError::InvalidSessionState(e))?
            .ok_or(AppError::AuthenticationFailed)?; // Handle case where user_id is not in session

        if poll.is_closed {
            return Err(AppError::Poll(PollsError::PollClosed));
        }

        if poll.is_paused {
            return Err(AppError::Poll(PollsError::PollPaused));
        }

        if let Ok(false) = self.can_vote(user_id.clone(), poll_id.clone()).await {
            return Err(AppError::Poll(PollsError::AlreadyVoted));
        }

        let update_result = self
            .polls
            .update_one(
                mongodb::bson::doc! {
                    "pollId": &poll_id,
                    "options.optionId": option_id
                },
                mongodb::bson::doc! {
                    "$inc": {
                        "options.$.votes": 1
                    },
                    "$push": {
                        "votedBy": user_id
                    }
                },
            )
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if update_result.modified_count == 0 {
            return Err(AppError::Poll(PollsError::InvalidPollOption));
        }

        self.get_poll_by_id(poll_id)
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))
    }

    pub async fn close_poll(&self, poll_id: String) -> Result<PollResponseDTO, AppError> {
        let update_result = self
            .polls
            .update_one(
                mongodb::bson::doc! { "pollId": &poll_id },
                mongodb::bson::doc! { "$set": { "isClosed": true } },
            )
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if update_result.modified_count == 0 {
            return Err(AppError::Poll(PollsError::PollNotFound));
        }

        self.get_poll_by_id(poll_id)
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))
    }

    pub async fn reset_poll(&self, poll_id: String) -> Result<PollResponseDTO, AppError> {
        let poll = self
            .get_poll_by_id(poll_id.clone())
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))?;

        if poll.is_closed {
            return Err(AppError::Poll(PollsError::CannotModifyClosed));
        }

        let update_result = self
            .polls
            .update_one(
                mongodb::bson::doc! { "pollId": &poll_id },
                mongodb::bson::doc! {
                    "$set": {
                        "options.$[].votes": 0,
                        "isPaused": false
                    }
                },
            )
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if update_result.modified_count == 0 {
            return Err(AppError::Poll(PollsError::PollNotFound));
        }

        self.get_poll_by_id(poll_id)
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))
    }

    pub async fn get_poll_results(&self, poll_id: String) -> Result<PollResponseDTO, AppError> {
        self.get_poll_by_id(poll_id)
            .await?
            .ok_or(AppError::Poll(PollsError::PollNotFound))
    }
}
