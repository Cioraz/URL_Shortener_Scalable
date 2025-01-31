use crate::db::{retrieve_data, store_data, Data, Database};
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, Filter};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use base62;
use once_cell::sync::Lazy;

const EPOCH: i64 = 1609459200000; // Custom epoch (e.g., 2021-01-01)
const NODE_ID_BITS: i64 = 10;
const SEQUENCE_BITS: i64 = 12;

const MAX_NODE_ID: i64 = (1 << NODE_ID_BITS) - 1;
const MAX_SEQUENCE: i64 = (1 << SEQUENCE_BITS) - 1;

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
pub struct SnowflakeGenerator {
    node_id: i64,
    last_timestamp: AtomicI64,
    sequence: AtomicI64,
}

impl SnowflakeGenerator {
    pub fn new(node_id: i64) -> Self {
        assert!(node_id >= 0 && node_id <= MAX_NODE_ID, "Node ID must be between 0 and {}", MAX_NODE_ID);
        SnowflakeGenerator {
            node_id,
            last_timestamp: AtomicI64::new(0),
            sequence: AtomicI64::new(0),
        }
    }

    fn timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock moved backwards")
            .as_millis() as i64
            - EPOCH
    }

    pub fn generate(&self) -> i64 {
        let mut timestamp = Self::timestamp();
        let last = self.last_timestamp.load(Ordering::Acquire);

        if timestamp < last {
            panic!("Clock moved backwards");
        }

        if timestamp == last {
            let seq = self.sequence.fetch_add(1, Ordering::SeqCst) & MAX_SEQUENCE;
            if seq == 0 {
                while timestamp <= last {
                    timestamp = Self::timestamp();
                }
            }
        } else {
            self.sequence.store(0, Ordering::Relaxed);
        }

        self.last_timestamp.store(timestamp, Ordering::Release);

        (timestamp << (NODE_ID_BITS + SEQUENCE_BITS))
            | (self.node_id << SEQUENCE_BITS)
            | self.sequence.load(Ordering::Relaxed)
    }
}

pub fn generate_short_url_id(_long_url: &str) -> String {
    static GENERATOR: once_cell::sync::Lazy<SnowflakeGenerator> = 
        once_cell::sync::Lazy::new(|| SnowflakeGenerator::new(1));
    
    let snowflake_id = GENERATOR.generate();
    base62::encode(snowflake_id as u64)[0..7].to_string()
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
