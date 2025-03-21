use crate::db::{retrieve_data, store_data, Data, Database};
use rand::seq::IteratorRandom;
use ring::digest::{Context, SHA256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, Filter};

/// Extract Redis connection from the Arc<Mutex> and pass it into the handler functions.
pub fn with_db(
    db: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

/// Handle the generation of short URLs, storing the information in Redis.
pub async fn handle_generate_url(
    key: String,
    body: serde_json::Value,
    redis_connection: Arc<Mutex<redis::aio::MultiplexedConnection>>,
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
    let short_url_id = generate_short_url_id(long_url);

    // Store metadata in Redis (with expiration TTL of 30 seconds)
    let data = Data {
        creation_data: chrono::Local::now().to_rfc3339(),
        shortened_url: format!("http://localhost/{}", &short_url_id),
        long_url: long_url.to_string(),
        ttl: 30,
    };

    if let Err(e) = store_data(redis_connection, short_url_id.clone(), data).await {
        eprintln!("Database error: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "error": "DATABASE_ERROR" })),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let response_body = serde_json::json!({
        "status": "success",
        "short_url": short_url_id
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&response_body),
        StatusCode::OK,
    ))
}

/// Generate a unique short URL from the long URL.
pub fn generate_short_url_id(long_url: &str) -> String {
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
    redis_connection: Arc<Mutex<redis::aio::MultiplexedConnection>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Retrieve the short URL from request parameters
    let short_url = params.get("short_url").cloned().unwrap_or_default();

    if let Some(data) = retrieve_data(redis_connection, &short_url).await {
        let now = chrono::Local::now();
        let expiration_time = chrono::DateTime::parse_from_rfc3339(&data.creation_data).unwrap()
            + chrono::Duration::seconds(data.ttl.into());

        if now > expiration_time {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "status": "error",
                    "message": "Short URL has expired!"
                })),
                StatusCode::NOT_FOUND,
            ));
        }

        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "status": "success",
                "redirect_to": data.long_url
            })),
            StatusCode::OK,
        ));
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Short URL not found"
        })),
        StatusCode::NOT_FOUND,
    ))
}
