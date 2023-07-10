use serde_json::Value;
use sha256;
use std::{time::SystemTime, collections::HashMap, fs::{File, self, read_to_string}, io::{Write}, path::Path};

use crate::kibi::types::ContractTransactionData;

use super::{block::{BlockJson, Block}, instance::BlockchainInstance, types::{Kibi, KibiFields}};

pub fn hash_generator(data: String) -> String {
  return sha256::digest(data);
}

pub fn get_timestamp() -> u64 {
  let time = SystemTime::now();
  let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
  duration.as_secs()
}

/**
 * Converts Block (stringified JSON transactions) to BlockJson data (parse JSON transactions)
 */
pub fn block_to_blockjson(block: Block) -> BlockJson {
  // decode transactions
  let mut transaction_json: Vec<Value> = vec![];

  for transaction in block.transactions {
    transaction_json.push(serde_json::from_str(transaction.as_str()).unwrap());
  }

  // create a BlockJson data
  BlockJson {
    index: block.index,
    nonce: block.nonce,
    timestamp: block.timestamp,
    hash: block.hash,
    prev_hash: block.prev_hash,
    // update with the decoded transactions (json format)
    transactions: transaction_json,
  }
}

pub fn save_current_block_hash (buf: &[u8]) -> Result<(), std::io::Error> {
  // create "data" dir
  fs::create_dir_all("data")?;

  let mut file = File::create("data/cur-block.inf")
    .expect("Error while writing block info");

  file.write_all(buf)
}

pub fn save_block (block: &Block) -> Result<(), std::io::Error> {
  // create "data" dir
  fs::create_dir_all("data")?;

  let file_name = format!("data/{block_hash}.blk", block_hash = block.hash);
  let mut file = File::create(file_name)
    .expect("Error while writing block info");

  let encoded_block = serde_json::to_string(&block).unwrap();

  file.write_all(encoded_block.as_bytes())
}

pub fn load_current_block () -> Option<Block> {
  let path_to_read = Path::new("data/cur-block.inf");
  let current_block_hash = read_to_string(path_to_read);

  if current_block_hash.is_err() {
    eprintln!("cur_block.inf file not found");
    return None;
  }

  let path_to_current_block = format!("data/{block_hash}.blk", block_hash = current_block_hash.unwrap());
  let current_block_data = read_to_string(path_to_current_block)
    .expect("Block hash not found");

  serde_json::from_str(&current_block_data).unwrap()
}

pub fn load_block (block_hash: String) -> Option<Block> {
  // Ignore block_hash = "0"
  if block_hash == "0".to_string() {
    return None;
  }

  let path_to_block = format!("data/{block_hash}.blk", block_hash = block_hash);
  
  let current_block_data = read_to_string(path_to_block)
    .expect("Block hash not found");

  if current_block_data == "0" {
    return None;
  }

  Some(serde_json::from_str(&current_block_data).unwrap())
}

/**
 * Get the most updated Kib fields info from chain
 */
pub fn get_kibi_from_chain () -> Kibi{
  let chain = BlockchainInstance::get_chain();

  for block in chain {

    // decode transactions
    let block_json = block_to_blockjson(block.to_owned());

    for tx in block_json.transactions {
      if tx["kib"].is_object() {
        let restored_kib: Kibi = serde_json::from_value(tx).unwrap();
        return restored_kib;
      }
    }
  }

  Kibi { kibi: KibiFields { accounts: HashMap::new() } }
}

/**
 * Get the most updated Contract fields info from chain
 */
pub fn get_contract_from_chain (contract_id: String) -> Option<ContractTransactionData>{
  let chain = BlockchainInstance::get_chain();

  for block in chain {

    // decode transactions
    let block_json = block_to_blockjson(block.to_owned());

    for tx in block_json.transactions {
      if tx["contract_id"].is_string() {
        let contract: ContractTransactionData = serde_json::from_value(tx).unwrap();
        if contract.contract_id == contract_id {
          return Some(contract);
        }
      }
    }
  }

  None
}

// Difficulty of PoW algorithm
// WARNING: changing the DIFFICULTY may break the blockchain hashes
pub const DIFFICULTY: usize = 2;

// Default depth (how many to load) of blocks (used to get the blocks)
pub const SEARCH_BLOCK_DEPTH: u64 = 1000;