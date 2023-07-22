use rocket::{get, serde::json::Json};

use crate::{
    core_tables::user_name::USER_NAME_TABLE_NAME,
    kibi::{
        blockchain::Blockchain,
        types::BasicResponse
    },
};

type Response = BasicResponse<String>;

#[get("/<user_name>/<db_access_key>")]
pub fn get(user_name: String, db_access_key: String) -> Json<Response> {
    // Blockchain
    let mut blockchain = Blockchain::new();

    // Check fields
    if user_name.is_empty()
        || db_access_key.is_empty()
    {
        return Json(Response {
            success: false,
            error_msg: "Invalid transaction data".to_string(),
            data: None,
        });
    }

    // UserNameTable should be stored using only the db_access_key + user_name + core_table_name, this is
    // because the system need to have the hability to check if the user_name is already
    // taken when a new user tries to create an account.
    let user_name_check_contract_id = sha256::digest(format!(
        "{db_key}{user_name}{core_table_name}",
        db_key = db_access_key,
        user_name = user_name,
        core_table_name = USER_NAME_TABLE_NAME
    ));

    // Get user_name from chain
    let user_name_record_opt = blockchain.get_last_transaction_under_contract_full_depth(
        user_name_check_contract_id.clone(),
        &db_access_key,
    );

    if user_name_record_opt.is_none() {
        return Json(Response {
            success: false,
            error_msg: "User not found".to_string(),
            data: None,
        });
    }

    return Json(Response {
        success: true,
        error_msg: "".to_string(),
        data: Some(user_name),
    });
}
