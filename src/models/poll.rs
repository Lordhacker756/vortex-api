use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

use crate::dtos::responses::{PollOptionResponseDTO, PollResponseDTO};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    pub pollId: String,
    /// References UserId in the main collection
    pub createdBy: String,
    pub name: String,
    pub isMulti: bool,  //Allow multi-select
    pub isPaused: bool, //Pause the poll
    pub isClosed: bool,
    pub startDate: DateTime,      //Allow scheduling in future
    pub endDate: DateTime,        //To close the poll
    pub options: Vec<PollOption>, // Embedded options
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PollOption {
    pub optionName: String,
    pub optionId: String,
    pub votes: i32,
}

impl Poll {
    pub fn to_response_dto(self) -> PollResponseDTO {
        PollResponseDTO {
            poll_id: self.pollId,
            created_by: self.createdBy,
            name: self.name,
            is_multi: self.isMulti,
            is_paused: self.isPaused,
            is_closed: self.isClosed,
            start_date: self.startDate.to_string(),
            end_date: self.endDate.to_string(),
            options: self
                .options
                .into_iter()
                .map(|opt| PollOptionResponseDTO {
                    option_id: opt.optionId,
                    option_name: opt.optionName,
                    votes: opt.votes,
                })
                .collect(),
        }
    }
}
