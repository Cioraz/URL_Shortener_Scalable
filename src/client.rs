use dotenv::dotenv;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn run_client(long_url: &str) {
    // Setup to access env vars
    dotenv().ok();
    let addr = std::env::var("SERVER_1").expect("SERVER_1 IP not set in .env!");
    match TcpStream::connect(&addr).await {
        Ok(mut stream) => {
            println!("Connected to server {}", &addr);

            // Send long URL to server
            if let Err(e) = stream.write_all(long_url.as_bytes()).await {
                eprintln!("Failed to write to server: {}", e);
                return;
            }
            println!("URL Sent from client: {}", long_url);

            // Read response from server
            let mut buffer = [0; 1024];
            match stream.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let short_url = String::from_utf8_lossy(&buffer[..n]);
                    println!("Received from server: {}", short_url);
                }
                Ok(_) => println!("Server closed connection!"),
                Err(e) => eprintln!("Failed to read from server: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to connect to server: {}", e),
    }
}
