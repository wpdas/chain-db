use rocket::{post, serde::json::Json};

use crate::kibi::{
    instance::BlockchainInstance,
    types::{ContractTransactionData, CreateAccountPayload, TransactionType, UserAccount},
    utils::get_timestamp,
};

#[post("/", format = "json", data = "<tx_data>")]
pub fn post(tx_data: Json<CreateAccountPayload>) -> String {
    // Check fields
    if tx_data.user_name.is_empty()
        || tx_data.password.is_empty()
        || tx_data.db_access_key.is_empty()
    {
        return "Invalid transaction data".to_string(); // 404
    }

    // User ID = Contract ID based on user info (for CoreUserAccountTable)
    let contract_id = sha256::digest(format!(
        "{db_key}{user_name}{user_pass}",
        db_key = tx_data.0.db_access_key,
        user_name = tx_data.0.user_name,
        user_pass = tx_data.0.password
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
            serde_json::from_value::<UserAccount>(user_account_check.data).unwrap();
        println!("User {:?} already exists", &user_check_data.user_name);

        return user_check_data.id;
    }

    let user_account = UserAccount {
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

    contract_id
}
