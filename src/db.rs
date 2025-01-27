use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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
pub async fn store_data(database: Database, key: String, data: Data) -> RedisResult<()> {
    let serialized_data = serde_json::to_string(&data).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Serialization error",
            e.to_string(),
        ))
    })?;

    let mut conn = database.lock().await;
    let _: () = conn.set_ex(&key, serialized_data, data.ttl.into()).await?;
    Ok(())
}

/// Retrieve data from Redis asynchronously
pub async fn retrieve_data(database: Database, key: &str) -> Option<Data> {
    let mut conn = database.lock().await;
    let serialized_data: String = conn.get(key).await.ok()?;
    serde_json::from_str(&serialized_data).ok()
}

/// Delete expired or invalid data
pub async fn delete_data(database: Database, key: &str) -> RedisResult<()> {
    let mut conn = database.lock().await;
    conn.del(key).await?;
    Ok(())
}

// Example usage of the database in your `handle_generate_url` function
// pub async fn handle_generate_url(
//     key: String,
//     body: serde_json::Value,
//     database: Database,
//     api_key: String,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     if key != api_key {
//         return Ok(warp::reply::with_status(
//             warp::reply::json(&serde_json::json!({
//                 "error": "UNAUTHORIZED"
//             })),
//             warp::http::StatusCode::UNAUTHORIZED,
//         ));
//     }

//     let long_url = body["long_url"].as_str().unwrap_or("");
//     if long_url.is_empty() {
//         return Ok(warp::reply::with_status(
//             warp::reply::json(&serde_json::json!({
//                 "error": "INVALID URL"
//             })),
//             warp::http::StatusCode::BAD_REQUEST,
//         ));
//     }

//     let short_url = generate_short_url(long_url);
//     println!("Short URL: {}, Long URL: {}", &short_url, long_url);

//     let data = Data {
//         creation_data: chrono::Local::now().to_rfc3339(),
//         shortened_url: format!("{}/{}", "localhost", &short_url),
//         long_url: long_url.to_string(),
//         ttl: 30,
//     };

//     // Store the data in Redis
//     if let Err(err) = store_data(database.clone(), short_url.clone(), data) {
//         eprintln!("Error storing data in Redis: {}", err);
//         return Ok(warp::reply::with_status(
//             warp::reply::json(&serde_json::json!({
//                 "error": "INTERNAL SERVER ERROR"
//             })),
//             warp::http::StatusCode::INTERNAL_SERVER_ERROR,
//         ));
//     }

//     let response_body = serde_json::json!({
//         "status": "success",
//         "short_url": short_url
//     });

//     if let Ok(response_string) = serde_json::to_string(&response_body) {
//         println!("Generated response: {}", response_string);
//     }

//     let response = warp::reply::json(&response_body);
//     Ok(warp::reply::with_status(response, warp::http::StatusCode::OK))
// }

// Example usage of the database in your `handle_redirect_url` function
// pub async fn handle_redirect_url(
//     params: HashMap<String, String>,
//     database: Database,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let short_url = params.get("short_url").cloned().unwrap_or_default();

//     if let Some(data) = retrieve_data(database.clone(), &short_url) {
//         let now = chrono::Local::now();
//         let expiration_time = chrono::DateTime::parse_from_rfc3339(&data.creation_data)
//             .unwrap()
//             + chrono::Duration::seconds(data.ttl.into());

//         if now > expiration_time {
//             return Ok(warp::reply::json(&serde_json::json!({
//                 "status": "error",
//                 "message": "Short URL has expired!"
//             })));
//         }

//         return Ok(warp::reply::json(&serde_json::json!({
//             "status": "success",
//             "redirect_to": data.long_url
//         })));
//     }

//     Ok(warp::reply::json(&serde_json::json!({
//         "status": "error",
//         "message": "Short URL not found"
//     })))
// }
