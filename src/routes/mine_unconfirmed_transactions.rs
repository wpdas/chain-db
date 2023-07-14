use rocket::get;

use crate::kibi::{blockchain::MineReturnOptions, instance::BlockchainInstance};

#[get("/")]
pub fn get() -> String {
    let result = BlockchainInstance::blockchain().mine();

    let (has_pending_transactions, new_block_index) = match result {
        MineReturnOptions::BOOL(value) => (value, 0),
        MineReturnOptions::I64(index) => (true, index),
    };

    if !has_pending_transactions {
        return "No transactions to mine".to_string();
    }

    "Block #".to_string() + &new_block_index.to_string() + " is mined."
}
