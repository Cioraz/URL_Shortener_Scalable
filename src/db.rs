use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
// use bb8::Pool;
// use bb8_redis::RedisConnectionManager;
// use dashmap::DashMap;

pub type Database = Arc<Mutex<redis::aio::MultiplexedConnection>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub creation_data: String,
    pub shortened_url: String,
    pub long_url: String,
    pub ttl: u32,
    
}


/// Create a new Redis database connection
pub async fn init_db() -> Database {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client");
    let connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    Arc::new(Mutex::new(connection))
}

/// Store data in Redis asynchronously
pub async fn store_data(database: Database, short_url_id: String, data: Data) -> RedisResult<()> {
    let serialized_data = serde_json::to_string(&data).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Serialization error",
            e.to_string(),
        ))
    })?;

    let mut conn = database.lock().await;
    let _: () = conn
        .set_ex(&short_url_id, serialized_data, data.ttl.into())
        .await?;
    Ok(())
}

/// Retrieve data from Redis asynchronously
pub async fn retrieve_data(database: Database, short_url_id: &str) -> Option<Data> {
    let mut conn = database.lock().await;
    let serialized_data: String = conn.get(short_url_id).await.ok()?;
    serde_json::from_str(&serialized_data).ok()
}

/// Delete expired or invalid data
pub async fn delete_data(database: Database, short_url_id: &str) -> RedisResult<()> {
    let mut conn = database.lock().await;
    conn.del(short_url_id).await?;
    Ok(())
}

