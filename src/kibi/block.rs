use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use crate::kibi::{utils::get_timestamp, crypto::Base64};

// #[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub struct Block {
  pub height: i64,
  pub nonce: i64,
  pub transactions: Vec<String>, // stringified JSON format (data)
  pub timestamp: u64,
  pub hash: String,
  pub prev_hash: String,
}

// struct to be used by client (server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockJson {
  pub height: i64,
  pub nonce: i64,
  pub transactions: Vec<Value>, // JSON format (data)
  pub timestamp: u64,
  pub hash: String,
  pub prev_hash: String,
}

impl Block {
  pub fn new(height: i64, prev_hash: String, hash: Option<String>, transactions: Option<Vec<String>>) -> Block {
    Block {
        height,
        transactions: transactions.unwrap_or(vec![]),
        timestamp: get_timestamp(),
        hash: hash.unwrap_or("0".to_string()),
        prev_hash,
        nonce: 0
    }
  }

  pub fn compute_hash(&mut self) -> String {
    // Update its hash (compute using the entire block data)
    // let stringified_block = serde_json::to_string(&self).unwrap();
    let stringified_block = Base64::encode(self.try_to_vec().unwrap());

    self.hash = sha256::digest(stringified_block);
    // Return the current hash
    return self.hash.to_string();
  }
}