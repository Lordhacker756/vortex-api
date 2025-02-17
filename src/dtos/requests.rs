use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterQuery {
    pub username: String,
}
