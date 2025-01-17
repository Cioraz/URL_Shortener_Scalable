use dotenv::dotenv;
use tokio::net::TcpListener;

pub async fn run_server() {
    dotenv().ok();
    let addr = std::env::var("SERVER_1").expect("SERVER_1 IP not set in .env!");
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed Binding Address!");
    println!("Server Running on {}", &addr);

    loop {
        let (_socket, client_addr) = listener
            .accept()
            .await
            .expect("Failed to accept any connection!");
        println!("New connection from {}", client_addr);
    }
}
