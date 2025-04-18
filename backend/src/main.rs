use dotenv::dotenv;
use handlers::with_db;
use std::collections::HashMap;
use warp::cors;
use warp::Filter;

mod db;
mod handlers;

use prometheus::{register_counter, register_histogram_vec, Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
struct PrometheusErrorWrapper(prometheus::Error);
impl warp::reject::Reject for PrometheusErrorWrapper {}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
    let database: db::Database = db::init_db().await;

    let generate_url_counter = register_counter!(
        "generate_url_requests_total",
        "Total number of generate_url requests"
    )
    .unwrap();

    let buckets = prometheus::linear_buckets(0.01, 0.05, 20).unwrap();
    let generate_url_duration = register_histogram_vec!(
        "http_request_duration_seconds",
        "Request duration for generate_url",
        &["endpoint"],
        buckets
    )
    .unwrap();

    let cors = cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "API-Key", "Authorization"])
        .build();

    // Route: /generate_url
    let db1 = Arc::clone(&database);
    let api_key_generate = api_key.clone();
    let generate_url = warp::path("generate_url")
        .and(warp::post())
        .and(warp::header::<String>("API-Key"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db1))
        .and_then(move |key, body: serde_json::Value, db| {
            generate_url_counter.inc();
            let histogram = generate_url_duration.with_label_values(&["generate_url"]);
            let timer = histogram.start_timer();
            let api_key = api_key_generate.to_string();
            let fut = handlers::handle_generate_url(key, body, db, api_key);
            async move {
                let result = fut.await;
                timer.observe_duration();
                result
            }
        });

    // Route: /custom_url
    let db2 = Arc::clone(&database);
    let api_key_custom = api_key.clone();
    let custom_url = warp::path("custom_url")
        .and(warp::post())
        .and(warp::header::<String>("API-Key"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db2))
        .and_then(move |key, body: serde_json::Value, db| {
            let api_key = api_key_custom.to_string();
            handlers::handle_custom_url(key, body, db, api_key)
        })
        .with(cors.clone());

    // Route: /dns_resolver/:short_url
    let db3 = Arc::clone(&database);
    let redirect_route = warp::path!("dns_resolver" / String)
        .and(with_db(db3))
        .and_then(|short_url: String, db| {
            let mut map = HashMap::new();
            map.insert("short_url".to_string(), short_url);
            handlers::handle_redirect_url(map, db)
        })
        .with(cors.clone());

    // Route: /ping
    let ping = warp::path("ping").map(|| warp::reply::json(&"pong"));

    // Route: /metrics
    let metrics = warp::path("metrics").and_then(|| async move {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        match encoder.encode(&metric_families, &mut buffer) {
            Ok(_) => {
                let output = String::from_utf8(buffer.clone()).unwrap();
                Ok::<_, warp::Rejection>(warp::reply::with_header(
                    output,
                    "Content-Type",
                    encoder.format_type(),
                ))
            }
            Err(e) => Err(warp::reject::custom(PrometheusErrorWrapper(e))),
        }
    });

    let api_routes = generate_url
        .or(custom_url)
        .or(ping)
        .or(metrics)
        .with(cors.clone());

    let routes = api_routes.or(redirect_route);

    let socket_addr: SocketAddr = "0.0.0.0:8000"
        .parse()
        .expect("Failed to parse socket address");

    println!("Server starting, listening on {}", socket_addr);
    warp::serve(routes).run(socket_addr).await;
}
