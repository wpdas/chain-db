use once_cell::sync::Lazy;

use super::{
    block::Block,
    blockchain::{Blockchain, MineReturnOptions},
    types::{ContractTransactionData, ContractTransactionDataJson},
};

/**
 * A blockchian instance to be used globally
 *
 * This was the way I found to use a Blockchain instance globally:
 * https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
 */
static mut BLOCKCHAIN: Lazy<Blockchain> = Lazy::new(|| Blockchain::new());

pub struct BlockchainInstance {}

// Should copy and pass forward all the methods
impl BlockchainInstance {
    pub fn chain(depth: u64) -> Vec<Block> {
        unsafe { BLOCKCHAIN.chain(depth) }
    }

    pub fn get_transactions_under_contract(
        contract_id: String,
        db_access_key: &String,
        depth: u64,
    ) -> Vec<ContractTransactionDataJson> {
        unsafe { BLOCKCHAIN.get_transactions_under_contract(contract_id, db_access_key, depth) }
    }

    pub fn get_transactions_under_contract_full_depth(
        contract_id: String,
        db_access_key: &String,
    ) -> Vec<ContractTransactionDataJson> {
        unsafe { BLOCKCHAIN.get_transactions_under_contract_full_depth(contract_id, db_access_key) }
    }

    pub fn get_last_transaction_data_under_contract(
        contract_id: String,
        db_access_key: &String,
        depth: u64,
    ) -> Option<ContractTransactionDataJson> {
        unsafe {
            BLOCKCHAIN.get_last_transaction_data_under_contract(contract_id, db_access_key, depth)
        }
    }

    pub fn get_last_transaction_under_contract_full_depth(
        contract_id: String,
        db_access_key: &String,
    ) -> Option<ContractTransactionDataJson> {
        unsafe {
            BLOCKCHAIN.get_last_transaction_under_contract_full_depth(contract_id, db_access_key)
        }
    }

    pub fn add_new_transaction(transaction: ContractTransactionData, db_access_key: &String) {
        unsafe { BLOCKCHAIN.add_new_transaction(transaction, db_access_key) }
    }

    pub fn mine() -> MineReturnOptions {
        unsafe { BLOCKCHAIN.mine() }
    }
}
