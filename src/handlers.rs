use crate::db;
use std::collections::HashMap;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
    Filter,
};

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
    if key
        != std::env::var("API_KEY")
            .expect("API_KEY wasnt provided!")
            .to_string()
    {
        return Ok(warp::reply::with_status(
            "UNAUTHORIZED",
            StatusCode::UNAUTHORIZED,
        ));
    }
    let long_url = body["long_url"].as_str().unwrap_or("");
    if long_url.is_empty() {
        return Ok(warp::reply::with_status(
            "INVALID URL",
            StatusCode::BAD_REQUEST,
        ));
    }
    let short_url = generate_short_url(long_url);
    database
        .lock()
        .unwrap()
        .insert(short_url.clone(), long_url.to_string());
    let response = warp::reply::json(&serde_json::json!({
        "short_url":short_url
    }));
    Ok(warp::reply::with_status(response, StatusCode::OK))
}

pub async fn handle_redirect_url(
    params: HashMap<String, String>,
    database: db::Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let short_url = params.get("short_url").cloned().unwrap_or_default();
    let db_lock = database.lock().unwrap();

    if let Some(long_url) = db_lock.get(&short_url) {
        if let Ok(uri) = warp::http::Uri::try_from(long_url.as_str()) {
            return Ok(warp::redirect::permanent(uri).into_response());
        }

        return Ok(
            warp::reply::with_status("Invalid URL", StatusCode::BAD_REQUEST).into_response(),
        );
    }

    Ok(reply::with_status("Not Found", StatusCode::NOT_FOUND).into_response())
}
