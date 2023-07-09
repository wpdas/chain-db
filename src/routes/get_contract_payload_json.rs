use rocket::{get, serde::json::Json};

use crate::kibi::{utils::get_contract_from_chain, types::{ContractTransactionData, ContractTransactionDataJson, TransactionType}};

#[get("/<id>")]
pub fn get(id: String) -> Json<ContractTransactionDataJson> {
  let contract_payload = get_contract_from_chain(id);

  let current_data = contract_payload.unwrap_or(ContractTransactionData { tx_type: TransactionType::NONE, contract_id: "".to_string(), timestamp: Some(0), data: "{}".to_string() });
  
  let json_data = ContractTransactionDataJson {
    tx_type: current_data.tx_type,
    contract_id: current_data.contract_id,
    timestamp: current_data.timestamp,
    data: serde_json::from_str(&current_data.data).unwrap()
  };

  Json(json_data)
}