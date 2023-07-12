use rocket::{post, serde::json::Json};

use crate::kibi::{
  types::{ContractTransactionData, SecureContractTransactionData},
  utils::get_timestamp,
  instance::BlockchainInstance
};

// TIP: '_ can be used to set the type as "unknown"

#[post("/<mine>", format="json", data="<tx_data>")]
pub fn post(tx_data: Json<SecureContractTransactionData>, mine: u8) -> &'static str {
  // Check fields
  if tx_data.contract_id.is_empty() || tx_data.db_access_key.is_empty() {
    return "Invalid transaction data" // 404
  }

  // create a basic contract transaction
  let transaction = ContractTransactionData {
    tx_type: tx_data.0.tx_type,
    contract_id: tx_data.0.contract_id,
    timestamp: Some(get_timestamp()),
    data: tx_data.0.data
  };

  //Register encripted transaction
  let stringified_tx_data = serde_json::to_string(&transaction).unwrap();
  // let encripted_tx_data = encript(stringified_tx_data);
  // TODO: Encrypt the tx_data here

  BlockchainInstance::blockchain().add_new_transaction(stringified_tx_data);

  // Should mine?
  if mine == 1 {
    BlockchainInstance::blockchain().mine();
  }

  "Success"
}