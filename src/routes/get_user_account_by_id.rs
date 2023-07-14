use rocket::{get, serde::json::Json};

use crate::kibi::{
    instance::BlockchainInstance,
    types::UserAccount,
};

#[get("/<user_id>/<db_access_key>")]
pub fn get(user_id: String, db_access_key: String) -> Json<UserAccount> {

    let user_check_contract_payload =
        BlockchainInstance::get_last_transaction_under_contract_full_depth(
            user_id,
            &db_access_key,
        );

    if user_check_contract_payload.is_some() {
        let tx = user_check_contract_payload.unwrap();
        let tx_data = serde_json::from_value::<UserAccount>(tx.data).unwrap();
        return Json(tx_data);
    }

    Json(
        UserAccount {
            id: "".to_string(),
            user_name: "".to_string(),
            units: 0,
        }
    )
}
