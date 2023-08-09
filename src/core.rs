use std::{collections::HashMap, net::SocketAddr};

use bytes::Bytes;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{message::BroadcastMessage, network::SimpleSender};

#[cfg(test)]
#[path = "tests/core_tests.rs"]
pub mod core_tests;

pub struct Core {
    id: usize,
    nodes: Vec<SocketAddr>,
    sender: SimpleSender,
    rx: Receiver<BroadcastMessage>,
    faults: usize,
    leader: usize,
    tx_term: Sender<u8>,
    rx_term: Receiver<u8>,
    input: Bytes,
    echo_map: HashMap<Vec<u8>, usize>,
    ready_map: HashMap<Vec<u8>, usize>,
    ready: bool,
}

impl Core {
    pub fn spawn(
        id: usize,
        nodes: Vec<SocketAddr>,
        sender: SimpleSender,
        rx: Receiver<BroadcastMessage>,
        faults: usize,
        leader: usize,
        input: Bytes,
    ) {
        println!("{} spawning Core", id);
        // Channel used for terminating the core.
        let (tx_term, rx_term) = channel(1);
        tokio::spawn(async move {
            Self {
                id,
                nodes,
                sender,
                rx,
                faults,
                leader,
                tx_term,
                rx_term,
                input,
                echo_map: HashMap::new(),
                ready_map: HashMap::new(),
                ready: false,
            }
            .run()
            .await;
        });
    }

    async fn handle_value(&mut self, v: Vec<u8>) {
        println!(
            "Node {} received value {:?}",
            self.id,
            std::str::from_utf8(v.as_slice()).unwrap().to_string()
        );
        let m = BroadcastMessage::Echo(v);
        self.broadcast(m).await;
    }

    async fn handle_echo(&mut self, v: Vec<u8>) {
        self.echo_map.insert(
            v.clone(),
            1 + if self.echo_map.contains_key(&v.clone()) {
                self.echo_map[&v.clone()]
            } else {
                0
            },
        );
        if self.echo_map[&v] >= self.nodes.len() - self.faults && !self.ready {
            self.ready = true;
            let m = BroadcastMessage::Ready(v);
            self.broadcast(m).await;
        }
    }

    async fn handle_ready(&mut self, v: Vec<u8>) {
        self.ready_map.insert(
            v.clone(),
            1 + if self.ready_map.contains_key(&v.clone()) {
                self.ready_map[&v.clone()]
            } else {
                0
            },
        );
        if self.ready_map[&v.clone()] >= self.faults + 1 && !self.ready {
            self.ready = true;
            let m = BroadcastMessage::Ready(v.clone());
            self.broadcast(m).await;
        }

        if self.ready_map[&v.clone()] >= self.nodes.len() - self.faults {
            println!(
                "Node {} received enough READY messages. Outputting {:?}",
                self.id,
                std::str::from_utf8(v.as_slice()).unwrap().to_string()
            );
            self.tx_term.send(0).await.unwrap();
        }
    }

    /// Broadcast a given message to every node in the network.
    async fn broadcast(&mut self, m: BroadcastMessage) {
        let bytes = Bytes::from(bincode::serialize(&m).unwrap());
        self.sender.broadcast(self.nodes.clone(), bytes).await;
    }

    pub async fn run(&mut self) {
        // If we are leader multicast value
        if self.id == self.leader {
            println!("{} is leader!", self.id);
            let m = BroadcastMessage::Value(self.input.to_vec());
            self.broadcast(m).await;
        }

        // Listen to incoming messages and process them. Note: self.rx is the channel where we can
        // retreive data from the message receiver.
        loop {
            tokio::select! {
                Some(message) = self.rx.recv() => match message {
                    BroadcastMessage::Value(v) => self.handle_value(v).await,
                    BroadcastMessage::Echo(v) => self.handle_echo(v).await,
                    BroadcastMessage::Ready(v) => self.handle_ready(v).await,
                },
                Some(_) = self.rx_term.recv() => {
                    println!("Node {} terminating..", self.id);
                    return;
                }
            };
        }
    }
}
