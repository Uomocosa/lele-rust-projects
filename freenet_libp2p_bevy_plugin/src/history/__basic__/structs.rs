use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub lobby: String,
    pub chunk: u64,
    pub data: Vec<u8>,
}
