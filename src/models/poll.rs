use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    pub pollId: String,
    /// References UserId in the main collection
    pub createdBy: String,
    pub name: String,
    pub isMulti: bool, //Allow multi-select
    pub options: Option<Vec<PollOptions>>,
    pub isPaused: bool,      //Pause the poll
    pub startDate: DateTime, //Allow scheduling in future
    pub endDate: DateTime,   //To close the poll
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PollOptions {
    /// References pollId in the main collection
    pub pollId: String,
    pub optionName: String,
    pub votes: i32,
}
