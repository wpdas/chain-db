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

#[derive(Debug, Deserialize)]
pub struct CreateAccountPayload {
    pub db_access_key: String,
    pub user_name: String,
    pub password: String,
    pub units: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferUnitsData {
    pub tx_type: TransactionType,
    pub contract_id: String,
    pub db_access_key: String,
    pub from: String,
    pub to: String,
    pub units: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String, // Used to refer the user
    pub user_name: String,
    pub units: u64,
}
