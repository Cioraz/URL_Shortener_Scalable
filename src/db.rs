use bb8_redis::{bb8, redis::cmd, RedisConnectionManager};
use redis::{RedisError, RedisResult};
use serde::{Deserialize, Serialize};

pub type Pool = bb8::Pool<RedisConnectionManager>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub creation_data: String,
    pub shortened_url: String,
    pub long_url: String,
    pub ttl: u32,
}

/// Create a new Redis connection pool
pub async fn init_db() -> Pool {
    let manager = RedisConnectionManager::new("redis://127.0.0.1")
        .expect("Failed to create Redis connection manager");

    bb8::Pool::builder()
        .max_size(100) // Set the maximum number of connections in the pool
        .build(manager)
        .await
        .expect("Failed to create Redis connection pool")
}

/// Store data in Redis asynchronously
pub async fn store_data(pool: &Pool, short_url_id: String, data: Data) -> RedisResult<()> {
    let serialized_data = serde_json::to_string(&data).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Serialization error",
            e.to_string(),
        ))
    })?;

    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::IoError,
            "Failed to get connection from pool",
            e.to_string(),
        ))
    })?;

    // Use `cmd` API to execute the SETEX command
    cmd("SETEX")
        .arg(&short_url_id)
        .arg(data.ttl)
        .arg(serialized_data)
        .query_async(&mut *conn)
        .await
        .map_err(|e| {
            // Convert the error from cmd into a RedisError
            RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to execute SETEX command",
                e.to_string(),
            ))
        })?;
    Ok(())
}

/// Retrieve data from Redis asynchronously
pub async fn retrieve_data(pool: &Pool, short_url_id: &str) -> Option<Data> {
    let mut conn = pool.get().await.ok()?;

    // Use `cmd` API to execute the GET command
    let serialized_data: String = cmd("GET")
        .arg(short_url_id)
        .query_async(&mut *conn)
        .await
        .ok()?;

    serde_json::from_str(&serialized_data).ok()
}

/// Delete expired or invalid data
pub async fn delete_data(pool: &Pool, short_url_id: &str) -> RedisResult<()> {
    let mut conn = pool.get().await.map_err(|e| {
        // Convert bb8's pool error into a RedisError
        RedisError::from((
            redis::ErrorKind::IoError,
            "Failed to get connection from pool",
            e.to_string(),
        ))
    })?;

    // Use `cmd` API to execute the DEL command and handle errors explicitly
    cmd("DEL")
        .arg(short_url_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| {
            // Convert the error from cmd into a RedisError
            RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to execute DEL command",
                e.to_string(),
            ))
        })?;
    Ok(())
}
