use rocket::{get, serde::json::Json};

use crate::{
    core_tables::user_account::UserAccountTable,
    kibi::{types::BasicResponse, utils::get_user_account_by_id},
};

type Response = BasicResponse<UserAccountTable>;

#[get("/<user_name>/<user_pass>/<db_access_key>")]
pub fn get(user_name: String, user_pass: String, db_access_key: String) -> Json<Response> {
    // User ID = Contract ID based on user info (for CoreUserAccountTable)
    let contract_id = sha256::digest(format!(
        "{db_key}{user_name}{user_pass}",
        db_key = db_access_key,
        user_name = user_name,
        user_pass = user_pass,
    ));

    let user = get_user_account_by_id(contract_id, &db_access_key);
    if user.is_some() {
        return Json(Response {
            success: true,
            error_msg: "".to_string(),
            data: user,
        });
    }

    Json(Response {
        success: false,
        error_msg: "User not found".to_string(),
        data: None,
    })
}
