use rocket::{get, serde::json::Json};
use serde_json::json;

use crate::kibi::{utils::SEARCH_BLOCK_DEPTH, types::{ContractTransactionDataJson, TransactionType}, instance::BlockchainInstance};

#[get("/<id>")]
pub fn get(id: String) -> Json<Vec<ContractTransactionDataJson>> {
  let mut transactions = BlockchainInstance::blockchain()
    .get_transactions_under_contract(id, SEARCH_BLOCK_DEPTH);

  if transactions.len() == 0 {
    transactions.push(ContractTransactionDataJson { tx_type: TransactionType::NONE, contract_id: "".to_string(), timestamp: Some(0), data: json!("{}"), block_hash: "".to_string(), block_height: 0 });
  }

  Json(transactions)
}