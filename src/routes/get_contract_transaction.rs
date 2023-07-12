use rocket::{get, serde::json::Json};
use serde_json::json;

use crate::kibi::{utils::SEARCH_BLOCK_DEPTH, types::{ContractTransactionDataJson, TransactionType}, instance::BlockchainInstance};

#[get("/<id>")]
pub fn get(id: String) -> Json<ContractTransactionDataJson> {
  let contract_payload = BlockchainInstance::blockchain()
    .get_last_transaction_data_under_contract(id, SEARCH_BLOCK_DEPTH);

  let current_data = contract_payload.unwrap_or(ContractTransactionDataJson { tx_type: TransactionType::NONE, contract_id: "".to_string(), timestamp: Some(0), data: json!("{}"), block_hash: "".to_string(), block_height: 0 });

  Json(current_data)
}