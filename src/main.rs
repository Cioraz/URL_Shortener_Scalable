use dotenv::dotenv;
use handlers::with_db;
use std::collections::HashMap;
use warp::Filter;

mod db;
mod handlers;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Load API key from environment variables
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    // Initialize the Redis connection pool
    let redis_pool = db::init_db().await;

    // Define the `/generate_url` endpoint
    let generate_url = warp::path("generate_url")
        .and(warp::post())
        .and(warp::header::<String>("API-Key"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(redis_pool.clone())) // Pass the Redis pool to the handler
        .and_then(move |key, body: serde_json::Value, db| {
            let api_key = api_key.to_string();
            handlers::handle_generate_url(key, body, db, api_key)
        });

    // Define the `/redirect_url` endpoint
    let redirect_url = warp::path("redirect_url")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(with_db(redis_pool.clone())) // Pass the Redis pool to the handler
        .and_then(handlers::handle_redirect_url);

    // Combine all routes
    let routes = generate_url.or(redirect_url);

    // Configure server address and port from environment variables or defaults
    let host = std::env::var("HOST_ADDR_1").unwrap_or_else(|_| "127.0.0.1".to_string());
    let host_port = std::env::var("HOST_PORT_1")
        .unwrap_or_else(|_| "15555".to_string())
        .parse::<u16>()
        .expect("PORT must be a number!");
    let host_parts: Vec<u8> = host
        .split('.')
        .map(|x| x.parse::<u8>().expect("Host Part Must be a number!"))
        .collect();
    let host_address = Ipv4Addr::new(host_parts[0], host_parts[1], host_parts[2], host_parts[3]);
    let socket_addr = SocketAddr::new(IpAddr::V4(host_address), host_port);

    // Start Warp server
    warp::serve(routes).run(socket_addr).await;
}
