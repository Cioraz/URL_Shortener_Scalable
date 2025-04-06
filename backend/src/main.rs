use dotenv::dotenv;
use handlers::with_db;
use std::collections::HashMap;
use warp::Filter;
mod db;
mod handlers;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
    let database: db::Database = db::init_db().await;
    let db1 = Arc::clone(&database);

    let generate_url = warp::path("generate_url")
        .and(warp::post())
        .and(warp::header::<String>("API-Key"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(handlers::with_db(db1))
        .and_then(move |key, body: serde_json::Value, db| {
            let api_key = api_key.to_string();
            handlers::handle_generate_url(key, body, db, api_key)
        });

    let db2 = Arc::clone(&database);
    let redirect_url = warp::path("redirect_url")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(with_db(db2))
        .and_then(handlers::handle_redirect_url);

    let ping = warp::path("ping").map(|| warp::reply::json(&"pong"));
    
    let routes = generate_url.or(redirect_url).or(ping);


    // Explicitly bind to all interfaces on port 8000 to match Docker's internal port mapping
    let socket_addr: SocketAddr = "0.0.0.0:8000".parse().expect("Failed to parse socket address");
    
    println!("Server starting, listening on {}", socket_addr);
    
    warp::serve(routes).run(socket_addr).await;
}
