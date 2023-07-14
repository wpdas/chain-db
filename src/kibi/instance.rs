use once_cell::sync::Lazy;

use super::blockchain::Blockchain;

/**
 * A blockchian instance to be used globally
 *
 * This was the way I found to use a Blockchain instance globally:
 * https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
 */
static mut BLOCKCHAIN: Lazy<Blockchain> = Lazy::new(|| Blockchain::new());

pub struct BlockchainInstance {}

impl BlockchainInstance {
    pub fn blockchain() -> Blockchain {
        unsafe { BLOCKCHAIN.clone() }
    }
}
