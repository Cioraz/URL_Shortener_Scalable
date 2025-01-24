use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use redis::{Commands, RedisResult};
use serde::{Deserialize, Serialize};
use crate::handlers::generate_short_url;
pub type Database = Arc<Mutex<redis::Client>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub creation_data: String, // Store as a formatted string for Redis
    pub shortened_url: String,
    pub long_url: String,
    pub ttl: u32,
}


/// Create a new Redis database connection
pub fn init_db() -> Arc<Mutex<redis::Connection>> {
    // Create a Redis client and establish a connection
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client");
    let connection = client.get_connection().expect("Failed to connect to Redis");
    Arc::new(Mutex::new(connection))
}

pub async fn get_redis_connection(database: &Database) -> redis::Connection {
    let client = database.lock().unwrap();
    client.get_connection().expect("Failed to connect to Redis")
}

/// Store data in Redis
pub fn store_data(database: Database, key: String, data: Data) -> RedisResult<()> {
    // Lock the database to ensure thread safety
    let client = database.lock().unwrap();
    let mut conn = client.get_connection()?; // Get the Redis connection

    let serialized_data = serde_json::to_string(&data)
        .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization error", e.to_string())))?;

    // Store the serialized data with a TTL in Redis
    conn.set_ex::<_, _, ()>(key, serialized_data, data.ttl.into())?;
    Ok(())
}


/// Retrieve data from Redis
pub fn retrieve_data(database: Database, key: &str) -> Option<Data> {
    let client = database.lock().unwrap();
    let mut conn = client.get_connection().ok()?;

    let serialized_data: String = conn.get(key).ok()?;
    serde_json::from_str(&serialized_data).ok()
}

/// Delete expired or invalid data (optional utility)
pub fn delete_data(database: Database, key: &str) -> RedisResult<()> {
    let client = database.lock().unwrap();
    let mut conn = client.get_connection()?;

    let _: () = conn.del(key)?;
    Ok(())
}

/// Example usage of the database in your `handle_generate_url` function
pub async fn handle_generate_url(
    key: String,
    body: serde_json::Value,
    database: Database,
    api_key: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    if key != api_key {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "UNAUTHORIZED"
            })),
            warp::http::StatusCode::UNAUTHORIZED,
        ));
    }

    let long_url = body["long_url"].as_str().unwrap_or("");
    if long_url.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "INVALID URL"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let short_url = generate_short_url(long_url);
    println!("Short URL: {}, Long URL: {}", &short_url, long_url);

    let data = Data {
        creation_data: chrono::Local::now().to_rfc3339(),
        shortened_url: format!("{}/{}", "localhost", &short_url),
        long_url: long_url.to_string(),
        ttl: 30,
    };

    // Store the data in Redis
    if let Err(err) = store_data(database.clone(), short_url.clone(), data) {
        eprintln!("Error storing data in Redis: {}", err);
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "INTERNAL SERVER ERROR"
            })),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let response_body = serde_json::json!({
        "status": "success",
        "short_url": short_url
    });

    if let Ok(response_string) = serde_json::to_string(&response_body) {
        println!("Generated response: {}", response_string);
    }

    let response = warp::reply::json(&response_body);
    Ok(warp::reply::with_status(response, warp::http::StatusCode::OK))
}

/// Example usage of the database in your `handle_redirect_url` function
pub async fn handle_redirect_url(
    params: HashMap<String, String>,
    database: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let short_url = params.get("short_url").cloned().unwrap_or_default();

    if let Some(data) = retrieve_data(database.clone(), &short_url) {
        let now = chrono::Local::now();
        let expiration_time = chrono::DateTime::parse_from_rfc3339(&data.creation_data)
            .unwrap()
            + chrono::Duration::seconds(data.ttl.into());

        if now > expiration_time {
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": "Short URL has expired!"
            })));
        }

        return Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "redirect_to": data.long_url
        })));
    }

    Ok(warp::reply::json(&serde_json::json!({
        "status": "error",
        "message": "Short URL not found"
    })))
}
