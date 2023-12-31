use rocket::{get, serde::json::Json};
use serde_json::json;

use crate::kibi::{
    blockchain::Blockchain,
    types::{ContractTransactionDataJson, TransactionType},
};

#[get("/<contract_id>/<db_access_key>")]
pub fn get(contract_id: String, db_access_key: String) -> Json<ContractTransactionDataJson> {
    // Blockchain
    let blockchain = Blockchain::new();

    let contract_payload =
        blockchain.get_last_transaction_under_contract_full_depth(contract_id, &db_access_key);

    let current_data = contract_payload.unwrap_or(ContractTransactionDataJson {
        tx_type: TransactionType::NONE,
        contract_id: "".to_string(),
        timestamp: Some(0),
        data: json!("{}"),
        block_hash: "".to_string(),
        block_height: 0,
    });

    Json(current_data)
}
