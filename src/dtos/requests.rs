use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RegisterQuery {
    pub username: String,
}

#[derive(Deserialize, Clone)]
pub struct ResultQueryParams {
    pub live: Option<bool>,
    pub authToken: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone)]
pub struct VoteQueryParam {
    pub optionId: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone, Debug)]
pub struct CreatePollDTO {
    pub name: String,
    pub isMulti: bool,
    pub startDate: DateTime<Utc>,
    pub endDate: DateTime<Utc>,
    pub createdBy: String,

    pub options: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone)]
pub struct UpdatePollDTO {
    pub name: String,
    pub isMulti: bool,
    pub startDate: DateTime<Utc>,
    pub endDate: DateTime<Utc>,
}
