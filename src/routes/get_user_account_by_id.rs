use rocket::{get, serde::json::Json};

use crate::{
    core_tables::user_account::UserAccountTable,
    kibi::{types::BasicResponse, utils::get_user_account_by_id},
};

type Response = BasicResponse<UserAccountTable>;

#[get("/<user_id>/<db_access_key>")]
pub fn get(user_id: String, db_access_key: String) -> Json<Response> {
    let user = get_user_account_by_id(user_id, &db_access_key);
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
