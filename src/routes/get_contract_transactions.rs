use rocket::{get, serde::json::Json};
use serde_json::json;

use crate::kibi::{
    instance::BlockchainInstance,
    types::{ContractTransactionDataJson, TransactionType}
};

#[get("/<contract_id>/<db_access_key>/<depth>")]
pub fn get(contract_id: String, db_access_key: String, depth: u64) -> Json<Vec<ContractTransactionDataJson>> {
    let mut transactions = BlockchainInstance::get_transactions_under_contract(
        contract_id,
        &db_access_key,
        depth,
    );

    if transactions.len() == 0 {
        transactions.push(ContractTransactionDataJson {
            tx_type: TransactionType::NONE,
            contract_id: "".to_string(),
            timestamp: Some(0),
            data: json!("{}"),
            block_hash: "".to_string(),
            block_height: 0,
        });
    }

    Json(transactions)
}
