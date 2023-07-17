use sha256;
use std::{
    fs::{self, read_to_string, File},
    io::Write,
    path::Path,
    time::SystemTime,
};

use crate::core_tables::user_account::UserAccountTable;

use super::{block::Block, instance::BlockchainInstance};

pub fn hash_generator(data: String) -> String {
    return sha256::digest(data);
}

pub fn get_timestamp() -> u64 {
    let time = SystemTime::now();
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    duration.as_secs()
}

pub fn save_current_block_hash(buf: &[u8]) -> Result<(), std::io::Error> {
    // create "data" dir
    fs::create_dir_all("data")?;

    let mut file = File::create("data/cur-block.inf").expect("Error while writing block info");

    file.write_all(buf)
}

pub fn save_block(block: &Block) -> Result<(), std::io::Error> {
    // create "data" dir
    fs::create_dir_all("data")?;

    let file_name = format!("data/{block_hash}.blk", block_hash = block.hash);
    let mut file = File::create(file_name).expect("Error while writing block info");

    // Encoda em json simples pois é menor o tamanho do arquivo guardando o bloco
    let block_json = serde_json::to_string(&block).unwrap();

    file.write_all(block_json.as_bytes())
}

pub fn get_current_block_hash() -> Option<String> {
    let path_to_read = Path::new("data/cur-block.inf");
    let current_block_hash = read_to_string(path_to_read);

    if current_block_hash.is_err() {
        eprintln!("cur_block.inf file not found");
        return None;
    }

    let path_to_current_block = format!("{block_hash}", block_hash = current_block_hash.unwrap());

    Some(path_to_current_block)
}

pub fn load_current_block() -> Option<Block> {
    let path_to_current_block = get_current_block_hash();
    if path_to_current_block.is_none() {
        eprintln!("cur_block.inf file not found");
        return None;
    }
    let current_block_data =
        load_block(path_to_current_block.unwrap()).expect("Block hash not found");

    Some(current_block_data)
}

pub fn load_block(block_hash: String) -> Option<Block> {
    // Ignore block_hash = "0"
    if block_hash == "0".to_string() {
        return None;
    }

    let path_to_block = format!("data/{block_hash}.blk", block_hash = block_hash);
    let current_block_data = read_to_string(path_to_block).expect("Block hash not found");
    if current_block_data == "0" {
        return None;
    }

    Some(serde_json::from_str(&current_block_data).unwrap())
}

/**
 * Fetch the user account by its ID (contract_id for the given user)
 */
pub fn get_user_account_by_id(user_id: String, db_access_key: &String) -> Option<UserAccountTable> {
    let user_check_contract_payload =
        BlockchainInstance::get_last_transaction_under_contract_full_depth(user_id, db_access_key);

    if user_check_contract_payload.is_some() {
        let tx = user_check_contract_payload.unwrap();
        let tx_data = serde_json::from_value::<UserAccountTable>(tx.data).unwrap();
        return Some(tx_data);
    }

    None
}

// Difficulty of PoW algorithm
// WARNING: changing the DIFFICULTY may break the blockchain hashes
pub const DIFFICULTY: usize = 2;

// Default depth (how many to load) of blocks (used to get the blocks)
pub const SEARCH_BLOCK_DEPTH: u64 = 1000;

// Use AesEcb Encryption / Decryption (default = false)
pub const USE_AESECB_ENCRYPTION: bool = false;