use url_shortener::db::{self, Data};
use url_shortener::handlers::{handle_generate_url, handle_redirect_url};
use warp::http::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use warp::reply::Reply;

// Initialize a test Redis database connection
async fn init_test_db() -> db::Database {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client");
    let connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    Arc::new(Mutex::new(connection))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_handle_generate_url() {
        let db = init_test_db().await;
        let api_key = "test_api_key".to_string();

        // Prepare the request body
        let body = json!({
            "long_url": "http://example.com/some/long/url"
        });

        // Simulate a request to generate a short URL
        let response = handle_generate_url(api_key.clone(), body, db.clone(), api_key).await.unwrap();
        let response = response.into_response();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["status"], "success");
        assert!(json["short_url"].is_string());
    }

    #[tokio::test]
    async fn test_handle_redirect_url() {
        let db = init_test_db().await;
        let short_url_id = "test_redirect_key".to_string();
        
        // Prepare and store data in Redis
        let data = Data {
            creation_data: chrono::Local::now().to_rfc3339(),
            shortened_url: format!("http://localhost/{}", short_url_id),
            long_url: "http://example.com/some/long/url".to_string(),
            ttl: 30,
        };

        db::store_data(db.clone(), short_url_id.clone(), data).await.unwrap();

        // Prepare parameters for redirecting
        let mut params = HashMap::new();
        params.insert("short_url".to_string(), short_url_id.clone());

        // Simulate a request to redirect based on the short URL
        let response = handle_redirect_url(params, db.clone()).await.unwrap();
        let response = response.into_response();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["status"], "success");
        assert_eq!(json["redirect_to"], "http://example.com/some/long/url");
    }

}
