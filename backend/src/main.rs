use dotenv::dotenv;
use handlers::with_db;
use std::collections::HashMap;
use warp::Filter;
mod db;
mod handlers;
use std::net::SocketAddr;
use std::sync::Arc;

use prometheus::{Encoder, TextEncoder, register_counter, Counter, HistogramVec, register_histogram_vec};
#[derive(Debug)]
struct PrometheusErrorWrapper(prometheus::Error);
impl warp::reject::Reject for PrometheusErrorWrapper {}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
    let database: db::Database = db::init_db().await;
    let db1 = Arc::clone(&database);

    let generate_url_counter = register_counter!(
        "generate_url_requests_total",
        "Total number of generate_url requests"
    ).unwrap();

    let buckets = prometheus::linear_buckets(0.01, 0.05, 20).unwrap();
    let generate_url_duration = register_histogram_vec!(
        "http_request_duration_seconds",
        "Request duration for generate_url",
        &["endpoint"],
        buckets
        ).unwrap();

    let generate_url = warp::path("generate_url")
        .and(warp::post())
        .and(warp::header::<String>("API-Key"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(handlers::with_db(db1))
        .and_then(move |key, body: serde_json::Value, db| {
            generate_url_counter.inc();
            let histogram = generate_url_duration.with_label_values(&["generate_url"]);
            let timer = histogram.start_timer(); // start the timer

            let api_key = api_key.to_string();
            let fut = handlers::handle_generate_url(key, body, db, api_key);

            async move {
                let result = fut.await;
                timer.observe_duration(); // stop + record the time
                result
            }
        });

    let db2 = Arc::clone(&database);
    let redirect_url = warp::path("redirect_url")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(with_db(db2))
        .and_then(handlers::handle_redirect_url);

    // For health check
    let ping = warp::path("ping").map(|| warp::reply::json(&"pong"));

    // Add a new route for exposing Prometheus metrics at /metrics.
    // This route will collect all registered metrics from the default registry.
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
                    encoder.format_type()
                ))
            },
            Err(e) => Err(warp::reject::custom(PrometheusErrorWrapper(e))),
        }
    });

    let cors = warp::cors()
        // Allow all origins for development; adjust this for production
        .allow_any_origin()
        // Allow headers required by your frontend
        .allow_header("Content-Type")
        .allow_header("API-Key")
        // Allow the methods you need
        .allow_methods(vec!["GET", "POST"]);
    
    let routes = generate_url.or(redirect_url).or(ping).or(metrics).with(cors);


    // Explicitly bind to all interfaces on port 8000 to match Docker's internal port mapping
    let socket_addr: SocketAddr = "0.0.0.0:8000".parse().expect("Failed to parse socket address");
    
    println!("Server starting, listening on {}", socket_addr);
    
    warp::serve(routes).run(socket_addr).await;
}
