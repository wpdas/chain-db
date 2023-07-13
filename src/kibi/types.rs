use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub enum TransactionType {
  NONE,
  ACCOUNT,
  CONTRACT,
  TRANSFER,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecureContractTransactionData {
  pub tx_type: TransactionType,
  pub contract_id: String,
  pub db_access_key: String,
  pub timestamp: Option<u64>,
  pub data: String,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Debug, Serialize, Deserialize)]
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
  pub block_hash: String,
  pub block_height: i64,
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