use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct Poll {
    pollId: String,
    /// References UserId in the main collection
    createdBy: String,
    name: String,
    isMulti: bool, //Allow multi-select
    options: Option<Vec<PollOptions>>,
    isPaused: bool,      //Pause the poll
    startDate: DateTime, //Allow scheduling in future
    endDate: DateTime,   //To close the poll
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct PollOptions {
    /// References pollId in the main collection
    pollId: String,
    optionName: String,
    votes: i32,
}
