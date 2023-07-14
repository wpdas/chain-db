use crate::kibi::utils::get_timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: i64,
    pub nonce: i64,
    pub transactions: Vec<String>, // stringified JSON format (data)
    pub timestamp: u64,
    pub hash: String,
    pub prev_hash: String,
}

impl Block {
    pub fn new(
        height: i64,
        prev_hash: String,
        hash: Option<String>,
        transactions: Option<Vec<String>>,
    ) -> Block {
        Block {
            height,
            transactions: transactions.unwrap_or(vec![]),
            timestamp: get_timestamp(),
            hash: hash.unwrap_or("0".to_string()),
            prev_hash,
            nonce: 0,
        }
    }

    pub fn compute_hash(&mut self) -> String {
        // Update its hash (compute using the entire block data)
        let stringified_block = serde_json::to_string(&self).unwrap();

        self.hash = sha256::digest(stringified_block);
        // Return the current hash
        return self.hash.to_string();
    }
}
