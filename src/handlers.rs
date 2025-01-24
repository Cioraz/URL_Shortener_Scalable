use chrono::prelude::*;
use http::Uri;
use rand::seq::IteratorRandom;
use ring::digest::{Context, SHA256};
use redis::{Commands, Connection};
use std::collections::HashMap;
use warp::{http::StatusCode, reply::Reply, Filter};
use std::sync::{Arc, Mutex};

/// Extract Redis connection from the Arc<Mutex> and pass it into the handler functions.
pub fn with_db(
    redis_connection: Arc<Mutex<redis::Connection>>,
) -> impl warp::Filter<Extract = (Arc<Mutex<redis::Connection>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || redis_connection.clone())
}
/// Handle the generation of short URLs, storing the information in Redis.
pub async fn handle_generate_url(
    key: String,
    body: serde_json::Value,
    redis_connection: Arc<Mutex<redis::Connection>>,
    api_key: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Verify API key authorization
    if key != api_key {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "error": "UNAUTHORIZED" })),
            StatusCode::UNAUTHORIZED,
        ));
    }

    // Validate long URL from request body
    let long_url = body["long_url"].as_str().unwrap_or("");
    if long_url.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "error": "INVALID URL" })),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Generate the short URL
    let short_url = generate_short_url(long_url);

    // Store metadata in Redis (with expiration TTL of 30 seconds)
    let data = serde_json::json!({
        "creation_date": chrono::Local::now().to_rfc3339(),
        "long_url": long_url,
        "ttl": 30
    });

    let mut conn = redis_connection.lock().unwrap();
    let _: () = conn
        .set_ex(&short_url, data.to_string(), 30)  // Set with expiration time (TTL)
        .map_err(|e| eprintln!("Redis error: {}", e))
        .unwrap_or_default();

    // Respond with the generated short URL
    let response_body = serde_json::json!({
        "status": "success",
        "short_url": short_url
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&response_body),
        StatusCode::OK,
    ))
}

/// Generate a unique short URL from the long URL.
pub fn generate_short_url(long_url: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(long_url.as_bytes());
    let hash = context.finish();
    let truncated_hash = u128::from_le_bytes(hash.as_ref()[0..16].try_into().unwrap());
    let base62_encoded = base62::encode(truncated_hash);

    let mut rng = rand::thread_rng();
    let short_url_id: String = base62_encoded
        .chars()
        .choose_multiple(&mut rng, 7)
        .into_iter()
        .collect();
    short_url_id
}

/// Handle the redirection based on the short URL, validating its expiration time in Redis.
pub async fn handle_redirect_url(
    params: HashMap<String, String>,
    redis_connection: Arc<Mutex<redis::Connection>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Retrieve the short URL from request parameters
    let short_url = params.get("short_url").cloned().unwrap_or_default();

    // Lock the Redis connection to fetch the short URL data
    let mut conn = redis_connection.lock().unwrap();
    let result: Option<String> = conn.get(&short_url).ok();

    if let Some(data) = result {
        // Deserialize the stored data
        let data: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
        let creation_date = chrono::DateTime::parse_from_rfc3339(data["creation_date"].as_str().unwrap_or("")).unwrap();
        let ttl = data["ttl"].as_i64().unwrap_or(0);

        // Check if the short URL has expired
        if chrono::Local::now() > creation_date + chrono::Duration::seconds(ttl) {
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": "Short URL has expired!"
            }))
            .into_response());
        }

        // If valid, redirect to the long URL
        let long_url = data["long_url"].as_str().unwrap_or_default();
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "redirect_to": long_url
        }))
        .into_response());
    }

    // If the short URL doesn't exist in Redis, return an error
    Ok(warp::reply::json(&serde_json::json!({
        "status": "error",
        "message": "Short URL not found"
    }))
    .into_response())
}

