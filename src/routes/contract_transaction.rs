use rocket::{post, serde::json::Json};

use crate::kibi::{
    blockchain::Blockchain,
    encryption::AesEcb,
    instance::BlockchainInstance,
    types::{ContractTransactionData, SecureContractTransactionData},
    utils::get_timestamp,
};

use borsh::BorshSerialize;
use std::str;

// TIP: '_ can be used to set the type as "unknown"

#[post("/<mine>", format = "json", data = "<tx_data>")]
pub fn post(tx_data: Json<SecureContractTransactionData>, mine: u8) -> &'static str {
    // Check fields
    if tx_data.contract_id.is_empty() || tx_data.db_access_key.is_empty() {
        return "Invalid transaction data"; // 404
    }

    // create a basic contract transaction
    let transaction = ContractTransactionData {
        tx_type: tx_data.0.tx_type,
        contract_id: tx_data.0.contract_id,
        timestamp: Some(get_timestamp()),
        data: tx_data.0.data,
    };

    // println!("AAAAAAAAAA: {:?}", transaction);

    // Register encripted transaction
    // let borsh_tx_data = transaction.try_to_vec().unwrap();
    // println!("SOFRIMENTO: {:?}", borsh_tx_data);
    // let stringified_borsh_tx_data = str::from_utf8(&borsh_tx_data)
    //   .unwrap()
    //   .to_string();

    // let borsh_tx_str_vec: Vec<String> = borsh_tx_data.iter()
    //   .map(|n| n.to_string())
    //   .collect();

    // let stringified_borsh_tx_data = borsh_tx_str_vec.join(",");
    // TODO: Continuar daqui

    // Encrypt the transaction (data) using AesEcb + db_access_key
    // let encrypted_tx_data = AesEcb::encode(&stringified_borsh_tx_data, &tx_data.0.db_access_key);

    // Register transaction
    // BlockchainInstance::blockchain().add_new_transaction(encrypted_tx_data);
    let mut blockchain = Blockchain {
        unconfirmed_transactions: vec![],
    };
    // blockchain.add_new_transaction(encrypted_tx_data);
    blockchain.add_new_transaction(transaction, &tx_data.0.db_access_key);

    // Should mine?
    if mine == 1 {
        // BlockchainInstance::blockchain().mine();
        blockchain.mine();
    }

    "Success"
}
