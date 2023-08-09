use super::*;
use futures::future::try_join_all;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[tokio::test]
async fn send_test() {
    // Run one TCP server.
    let address = "127.0.0.1:1234".parse::<SocketAddr>().unwrap();
    let message = "Hello, world!";
    let handle = listener(address, message.to_string());

    // Run the sender.
    let mut sender = SimpleSender::new();
    sender.send(address, message.into()).await;

    assert!(handle.await.is_ok())
}

#[tokio::test]
async fn broadcast_test() {
    // Run four TCP servers.
    let message = "Hello, world!";
    let (handles, addresses): (Vec<_>, Vec<_>) = (0..4)
        .map(|x| {
            let address = format!("127.0.0.1:123{}", x).parse::<SocketAddr>().unwrap();
            (listener(address, message.to_string()), address)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .unzip();

    // Run the sender.
    let mut sender = SimpleSender::new();
    sender.broadcast(addresses, message.into()).await;

    assert!(try_join_all(handles).await.is_ok());
}

pub fn listener(address: SocketAddr, expected: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(&address).await.unwrap();
        let (socket, _) = listener.accept().await.unwrap();
        let transport = Framed::new(socket, LengthDelimitedCodec::new());
        let (_, mut reader) = transport.split();
        match reader.next().await {
            Some(Ok(received)) => {
                assert_eq!(received, expected);
            }
            _ => panic!("Failed to receive network message"),
        }
    })
}
