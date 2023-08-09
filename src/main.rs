use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

mod core;
mod message;
mod network;
mod node;

#[tokio::main]
async fn main() {
    // Create 4 local ip addresses with different ports.
    let addresses = (0..4)
        .map(|x| format!("127.0.0.1:123{}", x).parse::<SocketAddr>().unwrap())
        .collect::<Vec<_>>();

    // Spawn 4 nodes.
    for i in 0..4 {
        let addresses = addresses.clone();
        tokio::spawn(async move {
            node::Node::new(i, addresses, 0).await;
        });
    }

    sleep(Duration::from_millis(5000)).await;
}
