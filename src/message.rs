use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum BroadcastMessage {
    Value(Vec<u8>),
    Echo(Vec<u8>),
    Ready(Vec<u8>),
}
