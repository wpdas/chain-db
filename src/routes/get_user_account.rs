use rocket::{get, serde::json::Json};

use crate::kibi::{
    instance::BlockchainInstance,
    types::UserAccount,
};

#[get("/<user_name>/<user_pass>/<db_access_key>")]
pub fn get(user_name: String, user_pass: String, db_access_key: String) -> Json<UserAccount> {

    // User ID = Contract ID based on user info (for CoreUserAccountTable)
    let contract_id = sha256::digest(format!(
        "{db_key}{user_name}{user_pass}",
        db_key = db_access_key,
        user_name = user_name,
        user_pass = user_pass,
    ));

    let user_check_contract_payload =
        BlockchainInstance::get_last_transaction_under_contract_full_depth(
            contract_id,
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
