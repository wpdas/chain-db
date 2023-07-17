use rocket::{post, serde::json::Json};

use crate::{
    core_tables::transfer_units::{TransferUnitsTable, TRANSFER_UNITS_TABLE_NAME},
    kibi::{
        encryption::Base64VecU8,
        instance::BlockchainInstance,
        types::{BasicResponse, ContractTransactionData, TransactionType, TransferUnitsPayload},
        utils::{get_timestamp, get_user_account_by_id},
    },
};

#[post("/", format = "json", data = "<tx_data>")]
pub fn post(tx_data: Json<TransferUnitsPayload>) -> Json<BasicResponse<String>> {
    // Check fields
    if tx_data.db_access_key.is_empty() || tx_data.from.is_empty() || tx_data.to.is_empty() {
        return Json(BasicResponse {
            success: false,
            error_msg: "Invalid transaction data".to_string(),
            data: None,
        });
    }

    let units = tx_data.0.units;

    // 1 - Check user from
    let user_from = get_user_account_by_id(tx_data.0.from, &tx_data.0.db_access_key);
    if user_from.is_none() {
        return Json(BasicResponse {
            success: false,
            error_msg: "Sender user not found".to_string(),
            data: None,
        });
    }
    let mut user_from_data = user_from.unwrap();

    // 2 - Check user to
    let user_to = get_user_account_by_id(tx_data.0.to, &tx_data.0.db_access_key);
    if user_to.is_none() {
        return Json(BasicResponse {
            success: false,
            error_msg: "Target user not found".to_string(),
            data: None,
        });
    }
    let mut user_to_data = user_to.unwrap();

    // 3 - Check if user From has enough units to transfer
    if user_from_data.units < tx_data.0.units {
        return Json(BasicResponse {
            success: false,
            error_msg: "Sender user does not have enough units".to_string(),
            data: None,
        });
    }

    // 4 - Move the units
    user_from_data.units -= &units;
    user_to_data.units += &units;

    // Dados de transferencia - ser executado
    // 1 - TransferUnitsData: from, to, units
    // 1.1 - FROM: Salva sob o contrato hash(db_access_key + user_from_id + "core-transfer-table");
    // [desta forma, é possivel buscar transacoes do usuario passando
    // o db_access_key + user_from_id.]
    // 1.2 - TO: Salva sob o contrato hash(db_access_key + user_to_id);
    // 2 - Cria transação para atualizacao de units para FROM;
    // 3 - Cria transaçao para atualizacao de units para para TO;

    // 1
    let transfer_units = TransferUnitsTable {
        from: user_from_data.id.clone(),
        to: user_to_data.id.clone(),
        units: units,
    };

    // 1.1
    // contract_id for transaction registry = hash(db_access_key + user_from_id + "core-transfer-table")
    let contract_id_tx_registry_from = sha256::digest(format!(
        "{db_access_key}{user_id}{core_table_name}",
        db_access_key = tx_data.0.db_access_key,
        user_id = user_from_data.id,
        core_table_name = TRANSFER_UNITS_TABLE_NAME
    ));
    BlockchainInstance::add_new_transaction(
        ContractTransactionData {
            tx_type: TransactionType::TRANSFER,
            contract_id: contract_id_tx_registry_from,
            timestamp: Some(get_timestamp()),
            data: serde_json::to_string(&transfer_units).unwrap(),
        },
        &tx_data.0.db_access_key,
    );

    // 1.2
    // contract_id for transaction registry = hash(db_access_key + user_to_id + "core-transfer-table")
    let contract_id_tx_registry_to = sha256::digest(format!(
        "{db_access_key}{user_id}{core_table_name}",
        db_access_key = tx_data.0.db_access_key,
        user_id = user_to_data.id,
        core_table_name = TRANSFER_UNITS_TABLE_NAME
    ));
    BlockchainInstance::add_new_transaction(
        ContractTransactionData {
            tx_type: TransactionType::TRANSFER,
            contract_id: contract_id_tx_registry_to,
            timestamp: Some(get_timestamp()),
            data: serde_json::to_string(&transfer_units).unwrap(),
        },
        &tx_data.0.db_access_key,
    );

    // 2
    BlockchainInstance::add_new_transaction(
        ContractTransactionData {
            tx_type: TransactionType::TRANSFER,
            contract_id: user_from_data.id.clone(),
            timestamp: Some(get_timestamp()),
            data: serde_json::to_string(&user_from_data).unwrap(),
        },
        &tx_data.0.db_access_key,
    );

    // 3
    BlockchainInstance::add_new_transaction(
        ContractTransactionData {
            tx_type: TransactionType::TRANSFER,
            contract_id: user_to_data.id.clone(),
            timestamp: Some(get_timestamp()),
            data: serde_json::to_string(&user_to_data).unwrap(),
        },
        &tx_data.0.db_access_key,
    );

    // Mine transactions
    BlockchainInstance::mine();

    Json(BasicResponse {
        success: true,
        error_msg: "".to_string(),
        data: None,
    })
}
