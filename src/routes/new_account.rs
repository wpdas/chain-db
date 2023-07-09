use rocket::{post, serde::json::Json};

use crate::kibi::{
  types::{NewAccountData},
  instance::BlockchainInstance, utils::{get_kibi_from_chain}, account::AccountId
};

#[post("/", format="json", data="<tx_data>")]
pub fn post(tx_data: Json<NewAccountData>) -> &'static str {

  let mut kib_data = get_kibi_from_chain();

  // Check fields
  if tx_data.account.is_empty() {
    return "Invalid transaction data" // 404
  }

  // Adds ".kib" at the end
  let new_account_name = tx_data.account.clone() + &".kib".to_string();

  // Check if this user is already registered
  if kib_data.kib.accounts.get(&new_account_name).is_some() {
    return "This account is already taken" // 404
  }

  kib_data.kib.accounts.insert(new_account_name.clone(), AccountId::parse(new_account_name));

  let stringified_tx_data = serde_json::to_string(&kib_data).unwrap();

  BlockchainInstance::add_new_transaction(stringified_tx_data);
  BlockchainInstance::mine();

  "Success"
}