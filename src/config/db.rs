use mongodb::{Client, Database};
use std::{env, sync::Arc};
use tracing::error;

pub async fn init_database() -> mongodb::error::Result<Arc<Database>> {
    let mongo_uri = env::var("MONGO_URI").map_err(|_| {
        error!("MONGO_URI not found in environment variables");
        mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "MONGO_URI not found",
        ))
    })?;

    let db_name = env::var("DATABASE_NAME").map_err(|_| {
        error!("DATABASE_NAME not found in environment variables");
        mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "DATABASE_NAME not found",
        ))
    })?;

    let client = Client::with_uri_str(&mongo_uri).await?;
    let database = client.database(&db_name);

    Ok(Arc::new(database))
}
