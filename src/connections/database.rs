use dotenv::dotenv;
use sqlx::{Error, PgPool};
use std::env;

pub async fn database_connection() -> Result<PgPool, Error> {
    // Load environment variables from .env file
    dotenv().ok();
    // Retrieve database URL from environment variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in the .env file");
    PgPool::connect(&database_url).await
}
