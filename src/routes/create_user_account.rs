use rocket::{post, serde::json::Json};

use crate::{
    core_tables::{
        user_account::{UserAccountTable, USER_ACCOUNT_TABLE_NAME},
        user_name::{UserNameTable, USER_NAME_TABLE_NAME},
    },
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

    // UserNameTable should be stored using only the db_access_key + core_table_name, this is
    // because the system need to have the hability to check if the user_name is already
    // taken when a new user tries to create an account.
    let user_name_check_contract_id = sha256::digest(format!(
        "{db_key}{core_table_name}",
        db_key = tx_data.0.db_access_key,
        core_table_name = USER_NAME_TABLE_NAME
    ));

    // Check if the user_name is available
    let user_name_record = BlockchainInstance::get_last_transaction_under_contract_full_depth(
        user_name_check_contract_id.clone(),
        &tx_data.0.db_access_key,
    );

    if user_name_record.is_some() {
        return Json(Response {
            success: false,
            error_msg: "This user name is already taken".to_string(),
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

    let user_account = UserAccountTable {
        id: contract_id.clone(),
        user_name: tx_data.0.user_name.clone(),
        units: tx_data.0.units.unwrap_or(0),
    };

    // New Account Transaction
    BlockchainInstance::add_new_transaction(
        ContractTransactionData {
            tx_type: TransactionType::ACCOUNT,
            contract_id: contract_id.clone(),
            timestamp: Some(get_timestamp()),
            data: serde_json::to_string(&user_account).unwrap(),
        },
        &tx_data.0.db_access_key,
    );

    let user_name_record = UserNameTable {
        user_name: tx_data.0.user_name,
        password_hint: tx_data.0.password_hint.unwrap_or("".to_string()),
    };

    // Register User Email + Pass Hint, Transaction
    BlockchainInstance::add_new_transaction(
        ContractTransactionData {
            tx_type: TransactionType::CONTRACT,
            contract_id: user_name_check_contract_id,
            timestamp: Some(get_timestamp()),
            data: serde_json::to_string(&user_name_record).unwrap(),
        },
        &tx_data.0.db_access_key,
    );

    // Mine transactions
    BlockchainInstance::mine();

    return Json(Response {
        success: false,
        error_msg: "".to_string(),
        data: Some(user_account),
    });
}
