use rocket::{get, serde::json::Json};
use serde_json::json;

use crate::kibi::{
    instance::BlockchainInstance,
    types::{ContractTransactionDataJson, TransactionType},
    utils::SEARCH_BLOCK_DEPTH,
};

#[get("/<contract_id>/<db_access_key>")]
pub fn get(contract_id: String, db_access_key: String) -> Json<Vec<ContractTransactionDataJson>> {
    let mut transactions = BlockchainInstance::blockchain().get_transactions_under_contract(
        contract_id,
        &db_access_key,
        SEARCH_BLOCK_DEPTH,
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
