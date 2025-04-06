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
    let try_urls = vec![
        "redis://redis:6379/",     // Docker Compose service name
        "redis://127.0.0.1:6379/", // Local fallback
    ];

    for url in try_urls {
        match redis::Client::open(url) {
            Ok(client) => match client.get_multiplexed_async_connection().await {
                Ok(conn) => {
                    println!("✅ Connected to Redis at: {}", url);
                    return Arc::new(Mutex::new(conn));
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect using {}: {}", url, e);
                }
            },
            Err(e) => {
                eprintln!("❌ Failed to create client using {}: {}", url, e);
            }
        }
    }

    panic!("🚨 Could not connect to Redis on any known address");
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

// Delete expired or invalid data
pub async fn delete_data(database: Database, short_url_id: &str) -> RedisResult<()> {
    let mut conn = database.lock().await;
    let _:() = conn.del(short_url_id).await?;
    Ok(())
}

