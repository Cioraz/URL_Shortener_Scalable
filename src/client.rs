use dotenv::dotenv;
use tokio::net::TcpStream;

pub async fn run_client() {
    dotenv().ok();
    let addr = std::env::var("SERVER_1").expect("SERVER_1 IP not set in .env!");
    match TcpStream::connect(&addr).await {
        Ok(_stream) => {
            println!("Connected to server {}", &addr);
        }
        Err(e) => eprintln!("Failed to connect to server: {}", e),
    }
}
