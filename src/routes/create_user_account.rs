use rocket::{post, serde::json::Json};

use crate::{
    core_tables::user_account::{UserAccountTable, USER_ACCOUNT_TABLE_NAME},
    kibi::{
        instance::BlockchainInstance,
        types::{BasicResponse, ContractTransactionData, CreateAccountPayload, TransactionType},
        utils::get_timestamp,
    },
};

type Response = BasicResponse<UserAccountTable>;

#[post("/", format = "json", data = "<tx_data>")]
pub fn post(tx_data: Json<CreateAccountPayload>) -> Json<Response> {
    // Check fields
    if tx_data.user_name.is_empty()
        || tx_data.password.is_empty()
        || tx_data.db_access_key.is_empty()
    {
        return Json(Response {
            success: false,
            error_msg: "Invalid transaction data".to_string(),
            data: None,
        });
    }

    // User ID = Contract ID based on user info (for CoreUserAccountTable)
    let contract_id = sha256::digest(format!(
        "{db_key}{user_name}{user_pass}{core_table_name}",
        db_key = tx_data.0.db_access_key,
        user_name = tx_data.0.user_name,
        user_pass = tx_data.0.password,
        core_table_name = USER_ACCOUNT_TABLE_NAME,
    ));

    // Check if user already exists, if so, just return its hash (id)
    let user_check_contract_payload =
        BlockchainInstance::get_last_transaction_under_contract_full_depth(
            contract_id.clone(),
            &tx_data.0.db_access_key,
        );
    if user_check_contract_payload.is_some() {
        let user_account_check = user_check_contract_payload.unwrap();
        let user_check_data =
            serde_json::from_value::<UserAccountTable>(user_account_check.data).unwrap();
        println!("User {:?} already exists", &user_check_data.user_name);

        return Json(Response {
            success: false,
            error_msg: "".to_string(),
            data: Some(user_check_data),
        });
    }

    let user_account = UserAccountTable {
        id: contract_id.clone(),
        user_name: tx_data.0.user_name,
        units: tx_data.0.units.unwrap_or(0),
    };

    // Create a basic contract transaction
    let transaction = ContractTransactionData {
        tx_type: TransactionType::ACCOUNT,
        contract_id: contract_id.clone(),
        timestamp: Some(get_timestamp()),
        data: serde_json::to_string(&user_account).unwrap(),
    };

    // Register transaction
    BlockchainInstance::add_new_transaction(transaction, &tx_data.0.db_access_key);
    BlockchainInstance::mine();

    return Json(Response {
        success: false,
        error_msg: "".to_string(),
        data: Some(user_account),
    });
}
