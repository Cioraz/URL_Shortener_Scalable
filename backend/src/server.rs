use dotenv::dotenv;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

pub async fn run_server() {
    dotenv().ok();
    let addr = std::env::var("SERVER_1").expect("SERVER_1 IP not set in .env!");
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed Binding Address!");
    println!("Server Running on {}", &addr);

    loop {
        // Accepting a new connection
        let (mut socket, client_addr) = listener
            .accept()
            .await
            .expect("Failed to accept any connection!");
        println!("New connection from {} client", client_addr);

        // Read data from client
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let long_url_client = String::from_utf8_lossy(&buffer[..n]);
                    // println!("Received URL from client: {}", long_url_client);

                    // Logic for Shortening

                    // Send response to client
                    let response = format!(
                        "Hello, client {} You sent: {}",
                        client_addr, long_url_client
                    );
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write to socket: {}", e);
                    }
                }
                Ok(_) => println!("Connection closed by client!"),
                Err(e) => eprintln!("Failed to read from socket: {}", e),
            }
        });
    }
}
