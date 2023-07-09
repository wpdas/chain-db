use rocket::{get, serde::json::Json};

use crate::kibi::{utils::get_contract_from_chain, types::{ContractTransactionData, TransactionType}};

#[get("/<id>")]
pub fn get(id: String) -> Json<ContractTransactionData> {
  let contract_payload = get_contract_from_chain(id);

  // serde_json::to_string(&contract_payload).unwrap_or("".to_string())
  Json(contract_payload.unwrap_or(ContractTransactionData { tx_type: TransactionType::NONE, contract_id: "".to_string(), timestamp: Some(0), data: "".to_string() }))
}