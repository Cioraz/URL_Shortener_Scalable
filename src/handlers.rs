use crate::db;
use rand::seq::IteratorRandom;
use ring::digest::{Context, SHA256};
use std::collections::HashMap;
use warp::{http::StatusCode, reply::Reply, Filter};

pub fn with_db(
    database: db::Database,
) -> impl Filter<Extract = (db::Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || database.clone())
}

pub async fn handle_generate_url(
    key: String,
    body: serde_json::Value,
    database: db::Database,
    api_key: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    if key != api_key {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "UNAUTHORIZED"
            })),
            StatusCode::UNAUTHORIZED,
        ));
    }
    let long_url = body["long_url"].as_str().unwrap_or("");
    if long_url.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "INVALID URL"
            })),
            StatusCode::BAD_REQUEST,
        ));
    }
    let short_url = generate_short_url(long_url);
    println!("Short URL: {}, Long URL: {}", &short_url, long_url);
    database
        .lock()
        .unwrap()
        .insert(short_url.clone(), long_url.to_string());
    let response_body = serde_json::json!({
        "short_url": short_url
    });

    // Serialize the JSON response body to a string for printing
    if let Ok(response_string) = serde_json::to_string(&response_body) {
        println!("Generated response: {}", response_string);
    }

    let response = warp::reply::json(&response_body);
    Ok(warp::reply::with_status(response, StatusCode::OK))
}

fn generate_short_url(long_url: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(long_url.as_bytes());
    let hash = context.finish();
    // So that it fits in u128 for later base62 encoding
    let truncated_hash = u128::from_le_bytes(hash.as_ref()[0..16].try_into().unwrap());
    let base62_encoded = base62::encode(truncated_hash);

    // Taking 7 characters from base62_encoded for random short URL id
    let mut rng = rand::thread_rng();
    let short_url_id: String = base62_encoded
        .chars()
        .choose_multiple(&mut rng, 7)
        .into_iter()
        .collect();
    short_url_id
}

pub async fn handle_redirect_url(
    params: HashMap<String, String>,
    database: db::Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let short_url = params.get("short_url").cloned().unwrap_or_default();
    let db_lock = database.lock().unwrap();

    if let Some(long_url) = db_lock.get(&short_url) {
        if let Ok(_uri) = warp::http::Uri::try_from(long_url.as_str()) {
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "redirect_to": long_url
            }))
            .into_response());
        }

        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Invalid URL format"
        }))
        .into_response());
    }

    Ok(warp::reply::json(&serde_json::json!({
        "status": "error",
        "message": "Short URL not found"
    }))
    .into_response())
}
