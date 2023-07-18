use rocket::{get, serde::json::Json};

use crate::{
    core_tables::transfer_units::{TransferUnitsTable, TRANSFER_UNITS_TABLE_NAME},
    kibi::{blockchain::Blockchain, types::BasicResponse},
};

type Response = BasicResponse<TransferUnitsTable>;

#[get("/<user_id>/<db_access_key>")]
pub fn get(user_id: String, db_access_key: String) -> Json<Response> {
    // Blockchain
    let blockchain = Blockchain::new();

    // contract_id for transaction registry = hash(db_access_key + user_from_id + "core-transfer-table")
    let contract_id_tx_registry = sha256::digest(format!(
        "{db_access_key}{user_id}{core_table_name}",
        db_access_key = db_access_key,
        user_id = user_id,
        core_table_name = TRANSFER_UNITS_TABLE_NAME
    ));

    let transfer_registry_opt = blockchain
        .get_last_transaction_under_contract_full_depth(contract_id_tx_registry, &db_access_key);

    if transfer_registry_opt.is_some() {
        let tx = transfer_registry_opt.unwrap();
        let tx_data = serde_json::from_value::<TransferUnitsTable>(tx.data).unwrap();
        return Json(Response {
            success: true,
            error_msg: "".to_string(),
            data: Some(tx_data),
        });
    }

    Json(Response {
        success: false,
        error_msg: "Transfer registry not found".to_string(),
        data: None,
    })
}
