use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
  NONE,
  ACCOUNT,
  CONTRACT,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractTransactionData {
  pub tx_type: TransactionType,
  pub contract_id: String,
  pub timestamp: Option<u64>,
  pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractTransactionDataJson {
  pub tx_type: TransactionType,
  pub contract_id: String,
  pub timestamp: Option<u64>,
  pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAccountData {
  pub account: String,
}

// KiBi
pub type KibiAccounts = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct KibiFields {
  pub accounts: KibiAccounts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Kibi {
  pub kibi: KibiFields
}