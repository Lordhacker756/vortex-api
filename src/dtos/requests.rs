use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RegisterQuery {
    pub username: String,
}
