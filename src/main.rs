mod client;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start the server on separate async task
    tokio::spawn(async {
        server::run_server().await;
    });

    // run client
    client::run_client("www.google.com").await;
    client::run_client("www.youtube.com").await;

    Ok(())
}
