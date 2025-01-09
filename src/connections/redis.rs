use redis::{aio::MultiplexedConnection, Client, RedisError};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn connect_to_redis() -> Result<Arc<Mutex<MultiplexedConnection>>, RedisError> {
    // Get the Redis URL from environment variables
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in .env file");

    // Create a Redis client
    let client = Client::open(redis_url).expect("Failed to create Redis client");

    // Use the recommended multiplexed connection
    let connection = client.get_multiplexed_async_connection().await?;

    Ok(Arc::new(Mutex::new(connection)))
}
