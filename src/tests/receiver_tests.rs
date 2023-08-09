use crate::message::BroadcastMessage;
use bytes::Bytes;
use futures::SinkExt;
use tokio::sync::mpsc::channel;
use tokio::time::{sleep, Duration};

use super::*;

#[tokio::test]
async fn receive_test() {
    // Spawn a receiver.
    let address = "127.0.0.1:1234".parse::<SocketAddr>().unwrap();
    let (tx, mut rx) = channel(100);
    MessageReceiver::spawn(address, tx);
    sleep(Duration::from_millis(50)).await;

    // Send a BroadcastMessage.
    let payload = "Hello, world!";
    let mes = BroadcastMessage::Ready(payload.as_bytes().to_vec());
    let bytes = Bytes::from(bincode::serialize(&mes).unwrap());
    let stream = TcpStream::connect(address).await.unwrap();
    let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
    transport.send(bytes.clone()).await.unwrap();

    let message = rx.recv().await;
    assert!(message.is_some());
    let received = message.unwrap();
    let res = match received {
        BroadcastMessage::Ready(v) => v == payload.as_bytes().to_vec(),
        _ => false,
    };
    assert!(res)
}
