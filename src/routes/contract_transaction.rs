use rocket::{post, serde::json::Json};

use crate::kibi::{
    instance::BlockchainInstance,
    types::{ContractTransactionData, SecureContractTransactionData},
    utils::get_timestamp,
};

#[post("/", format = "json", data = "<tx_data>")]
pub fn post(tx_data: Json<SecureContractTransactionData>) -> &'static str {
    // Check fields
    if tx_data.contract_id.is_empty() || tx_data.db_access_key.is_empty() {
        return "Invalid transaction data"; // 404
    }

    // create a basic contract transaction
    let transaction = ContractTransactionData {
        tx_type: tx_data.0.tx_type,
        contract_id: tx_data.0.contract_id,
        timestamp: Some(get_timestamp()),
        data: tx_data.0.data,
    };

    // Register transaction
    BlockchainInstance::add_new_transaction(transaction, &tx_data.0.db_access_key);
    BlockchainInstance::mine();

    "Success"
}
