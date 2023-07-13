use borsh::{BorshSerialize, BorshDeserialize};
use serde_json::Value;
use sha256;
use std::{time::SystemTime, fs::{File, self, read_to_string}, io::Write, path::Path};
use std::str;

use super::{block::{BlockJson, Block}, encryption::{Base64, AesEcb}};

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
pub fn block_to_blockjson(block: &Block) -> BlockJson {
  // decode transactions
  let mut transaction_json: Vec<Value> = vec![];

  for transaction in &block.transactions {
    transaction_json.push(serde_json::from_str(transaction.as_str()).unwrap());
  }

  // create a BlockJson data
  BlockJson {
    height: block.height,
    nonce: block.nonce,
    timestamp: block.timestamp,
    hash: block.hash.clone(),
    prev_hash: block.prev_hash.clone(),
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

  let encoded_block = Base64::encode(block.try_to_vec().unwrap());

  file.write_all(encoded_block.as_bytes())

  // --- NEW -- Criptografa o bloco inteiro -- NAO RECOMENDADO porque vai
  // deteriorar a forma de ler os blocos, ja que alguns, nao serao acessiveis
  // if db_access_key.is_some() {
  //   let encrypted_tx_data = AesEcb::encode(&encoded_block, db_access_key.unwrap());
  //   return file.write_all(encrypted_tx_data.as_bytes());
  // }

  // file.write_all(encoded_block.as_bytes())
  
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

    // Some(Block::try_from_slice(Base64::decode(current_block_data).as_ref()).unwrap())
  

  // serde_json::from_str(&current_block_data).unwrap()
  Some(Block::try_from_slice(Base64::decode(current_block_data).as_ref()).unwrap())
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

  // Some(serde_json::from_str(&current_block_data).unwrap())
  Some(Block::try_from_slice(Base64::decode(current_block_data).as_ref()).unwrap())
}

// Difficulty of PoW algorithm
// WARNING: changing the DIFFICULTY may break the blockchain hashes
pub const DIFFICULTY: usize = 2;

// Default depth (how many to load) of blocks (used to get the blocks)
pub const SEARCH_BLOCK_DEPTH: u64 = 1000;