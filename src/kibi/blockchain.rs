use crate::kibi::block::Block;
use crate::kibi::utils::{get_current_block_hash, hash_generator, DIFFICULTY};

use super::encryption::AesEcb;
use super::types::{ContractTransactionData, ContractTransactionDataJson};
use super::utils::{
    load_block, load_current_block, save_block, save_current_block_hash, SEARCH_BLOCK_DEPTH,
    USE_AESECB_ENCRYPTION,
};

// A way to inform the kinds of data for a given method
#[derive(Debug)]
pub enum MineReturnOptions {
    BOOL(bool),
    I64(i64),
}

#[derive(Clone, Debug)]
pub struct Blockchain {
    //   pub chain: Vec<Block>, // Talvez vai ser removido (os arquivos vivem localmente)
    pub unconfirmed_transactions: Vec<String>,
}

impl Blockchain {
    // Auto create an instance of Blockchain and init it
    pub fn new() -> Blockchain {
        let mut instance = Blockchain {
            // chain: Vec::new(),
            unconfirmed_transactions: Vec::new(),
        };
        instance.init();
        return instance;
    }

    // Init the blockchain with a genesis block
    fn init(&mut self) {
        // Check if there's any block
        let current_block_opt = load_current_block();

        if current_block_opt.is_none() {
            let hash_first = hash_generator("first_gen".to_string());
            let genesis_block = Block::new(0, "0".to_string(), Some(hash_first), None);

            // Save genesis block hash
            save_current_block_hash(genesis_block.hash.as_str().as_bytes()).unwrap();
            // Save genesis block
            save_block(&genesis_block).unwrap();
        }
    }

    // Get the last block
    fn last_block(&self) -> Block {
        let chain = self.chain(SEARCH_BLOCK_DEPTH).to_owned();
        chain.first().unwrap().clone()
    }

    /**
     * Adds the block to the chain after verification.
     *
     * Verification includes:
     * - Checking if the proof is valid.
     * - The previous_hash referred in the block and the hash of latest block
     *   in the chain match
     */
    fn add_block(&mut self, block: &Block, proof: String) -> bool {
        let previous_hash = &self.last_block().hash;

        if previous_hash != &block.prev_hash {
            return false;
        }

        if !self.is_valid_proof(&block, proof) {
            return false;
        }

        // Store the current block hash
        save_current_block_hash(block.hash.as_str().as_bytes()).unwrap();

        // Save the current block
        save_block(block).unwrap();

        true
    }

    /**
     * Returns all the blocks inside the blockchain.
     * THIS MAY BE DANGEROUS WHEN THERE'S BIG DATA. May exceed the memory allocation.
     * USE THIS TO DEBUG PURPOSES ONLY
     */
    pub fn chain(&self, depth: u64) -> Vec<Block> {
        let mut chain: Vec<Block> = vec![];

        let current_block = load_current_block().unwrap();
        chain.push(current_block.clone());

        let mut prev_block_hash = current_block.prev_hash.to_owned();

        let mut n = 1;

        while n < depth {
            let block_opt = load_block(prev_block_hash.to_owned());

            if block_opt.is_none() {
                break;
            }

            let block = block_opt.unwrap();
            chain.push(block.to_owned());

            // set the new prev_block_hash
            prev_block_hash = block.prev_hash.to_owned();

            n += 1;
        }

        return chain;
    }

    /**
     * Encrypt transaction using db_access_key
     */
    fn encrypt_transaction(
        &self,
        transaction: &ContractTransactionData,
        db_access_key: &String,
    ) -> String {
        // With AesEcb encryption - (WARNING: produces files 50% larger)
        if USE_AESECB_ENCRYPTION {
            // 1 - JSON serialize
            let borsh_tx_data = serde_json::to_string(transaction).unwrap();
            // 2 - Encrypt using AesEcb
            AesEcb::encode(&borsh_tx_data, &db_access_key)
        } else {
            // Without AesEcb encryption
            serde_json::to_string(transaction).unwrap()
        }
    }

    /**
     * Decrypt transaction using db_access_key
     */
    fn decrypt_transaction(
        &self,
        encrypted_data: String,
        db_access_key: &String,
    ) -> Option<ContractTransactionData> {
        // With AesEcb encryption - (WARNING: produces files 50% larger)
        if USE_AESECB_ENCRYPTION {
            // 1 - Dencrypt the transaction (data) using AesEcb + db_access_key
            let decoded_tx_opt = AesEcb::decode(&encrypted_data, &db_access_key);

            if decoded_tx_opt.is_none() {
                return None;
            }

            let decoded_tx: String = decoded_tx_opt.unwrap();

            // JSON deserialize
            let transaction = serde_json::from_str::<ContractTransactionData>(&decoded_tx).unwrap();
            Some(transaction)
        } else {
            // Without AesEcb encryption
            Some(serde_json::from_str::<ContractTransactionData>(&encrypted_data).unwrap())
        }
    }

    /**
     * Run over blocks to get the last transaction data under a specific contract
     * Gets all data/transactions inside the contract(living inside blocks) and try
     * to decrypt it using the given db_access_key, if it succeed, return the transaction
     *
     * Preffer to use this method than `get_transactions_under_contract` that's going
     * to fetch all transactions under a contract. This is heavy.
     *
     * WARNING: This is going to go over the entire chain till find the needed contract.
     */
    pub fn get_last_transaction_under_contract_full_depth(
        &self,
        contract_id: String,
        db_access_key: &String,
    ) -> Option<ContractTransactionDataJson> {
        let mut prev_block_hash = get_current_block_hash().unwrap();
        let mut n = 0;

        while n == 0 {
            let block_opt = load_block(prev_block_hash.to_owned());

            if block_opt.is_none() {
                n = 1;
                break;
            }

            let block = block_opt.unwrap();

            // Decode transactions
            for encrypted_tx in block.transactions {
                // Try to decrypt transaction using the given db_access_key
                let tx_opt = self.decrypt_transaction(encrypted_tx, db_access_key);
                if tx_opt.is_some() {
                    // Get the transaction obj
                    let tx = tx_opt.unwrap();

                    if tx.contract_id == contract_id {
                        let dec_tx_json = ContractTransactionDataJson {
                            tx_type: tx.tx_type,
                            contract_id: tx.contract_id,
                            timestamp: tx.timestamp,
                            data: serde_json::from_str(&tx.data).unwrap(),
                            block_hash: block.hash,
                            block_height: block.height,
                        };

                        return Some(dec_tx_json);
                    }
                }
            }

            // set the new prev_block_hash
            prev_block_hash = block.prev_hash;
        }

        None
    }

    /**
     * Run over blocks to get the transactions under a specific contract
     * Gets all data/transactions inside the contract(living inside blocks) and try
     * to decrypt it using the given db_access_key, if it succeed, return the transaction
     *
     * Preffer to use `get_last_transaction_data_under_contract` that's going to return
     * only one transaction/data (the most recent one)
     */
    pub fn get_transactions_under_contract(
        &self,
        contract_id: String,
        db_access_key: &String,
        depth: u64,
    ) -> Vec<ContractTransactionDataJson> {
        let mut transactions: Vec<ContractTransactionDataJson> = vec![];
        let mut prev_block_hash = get_current_block_hash().unwrap();
        let mut n = 0;

        while n < depth {
            let block_opt = load_block(prev_block_hash.to_owned());

            if block_opt.is_none() {
                break;
            }

            let block: Block = block_opt.unwrap();

            // Decode transactions
            for encrypted_tx in block.transactions {
                // Try to decrypt transaction using the given db_access_key
                let tx_opt = self.decrypt_transaction(encrypted_tx, db_access_key);
                if tx_opt.is_some() {
                    // Get the transaction obj
                    let tx = tx_opt.unwrap();

                    if tx.contract_id == contract_id {
                        // let dec_tx = serde_json::from_value::<ContractTransactionData>(tx).unwrap();

                        // create json version from dec_tx
                        let dec_tx_json = ContractTransactionDataJson {
                            tx_type: tx.tx_type,
                            contract_id: tx.contract_id,
                            timestamp: tx.timestamp,
                            data: serde_json::from_str(&tx.data).unwrap(),
                            block_hash: block.hash.clone(),
                            block_height: block.height,
                        };

                        transactions.push(dec_tx_json);
                    }
                }
            }

            // set the new prev_block_hash
            prev_block_hash = block.prev_hash;

            n += 1;
        }

        return transactions;
    }

    /**
     * Run over blocks to get the transactions under a specific contract
     * Gets all data/transactions inside the contract(living inside blocks) and try
     * to decrypt it using the given db_access_key, if it succeed, return the transaction
     *
     * Preffer to use `get_last_transaction_data_under_contract` that's going to return
     * only one transaction/data (the most recent one)
     *
     * WARNING: This is going to go over the entire chain till find the needed contract.
     */
    pub fn get_transactions_under_contract_full_depth(
        &self,
        contract_id: String,
        db_access_key: &String,
    ) -> Vec<ContractTransactionDataJson> {
        let mut transactions: Vec<ContractTransactionDataJson> = vec![];
        let mut prev_block_hash = get_current_block_hash().unwrap();
        let mut n = 0;

        while n == 0 {
            let block_opt = load_block(prev_block_hash.to_owned());

            if block_opt.is_none() {
                n = 1;
                break;
            }

            let block: Block = block_opt.unwrap();

            // Decode transactions
            for encrypted_tx in block.transactions {
                // Try to decrypt transaction using the given db_access_key
                let tx_opt = self.decrypt_transaction(encrypted_tx, db_access_key);
                if tx_opt.is_some() {
                    // Get the transaction obj
                    let tx = tx_opt.unwrap();

                    if tx.contract_id == contract_id {
                        // let dec_tx = serde_json::from_value::<ContractTransactionData>(tx).unwrap();

                        // create json version from dec_tx
                        let dec_tx_json = ContractTransactionDataJson {
                            tx_type: tx.tx_type,
                            contract_id: tx.contract_id,
                            timestamp: tx.timestamp,
                            data: serde_json::from_str(&tx.data).unwrap(),
                            block_hash: block.hash.clone(),
                            block_height: block.height,
                        };

                        transactions.push(dec_tx_json);
                    }
                }
            }

            // set the new prev_block_hash
            prev_block_hash = block.prev_hash;
        }

        return transactions;
    }

    /**
     * Run over blocks to get the last transaction data under a specific contract
     * Gets all data/transactions inside the contract(living inside blocks) and try
     * to decrypt it using the given db_access_key, if it succeed, return the transaction
     *
     * Preffer to use this method than `get_transactions_under_contract` that's going
     * to fetch all transactions under a contract. This is heavy.
     */
    pub fn get_last_transaction_data_under_contract(
        &self,
        contract_id: String,
        db_access_key: &String,
        depth: u64,
    ) -> Option<ContractTransactionDataJson> {
        let mut prev_block_hash = get_current_block_hash().unwrap();
        let mut n = 0;

        while n < depth {
            let block_opt = load_block(prev_block_hash.to_owned());

            if block_opt.is_none() {
                break;
            }

            let block = block_opt.unwrap();

            // Decode transactions
            for encrypted_tx in block.transactions {
                // Try to decrypt transaction using the given db_access_key
                let tx_opt = self.decrypt_transaction(encrypted_tx, db_access_key);
                if tx_opt.is_some() {
                    // Get the transaction obj
                    let tx = tx_opt.unwrap();

                    if tx.contract_id == contract_id {
                        // let dec_tx = serde_json::from_value::<ContractTransactionData>(tx).unwrap();

                        // create json version from dec_tx
                        let dec_tx_json = ContractTransactionDataJson {
                            tx_type: tx.tx_type,
                            contract_id: tx.contract_id,
                            timestamp: tx.timestamp,
                            data: serde_json::from_str(&tx.data).unwrap(),
                            block_hash: block.hash,
                            block_height: block.height,
                        };

                        return Some(dec_tx_json);
                    }
                }
            }

            // set the new prev_block_hash
            prev_block_hash = block.prev_hash;

            n += 1;
        }

        None
    }

    /**
     * Check if block_hash is valid hash of block and satisfies the difficulty criteria.
     */
    fn is_valid_proof(&self, block: &Block, block_hash: String) -> bool {
        // sets the difficulty chars. e.g.: 000 if DIFFICULTY is 3
        let difficulty_chars = "0".repeat(DIFFICULTY);

        block_hash.starts_with(difficulty_chars.as_str()) && block_hash == block.hash
    }

    /**
     * Function that tries different values of the nonce to get a hash
     * that satisfies our difficulty criteria.
     */
    fn proof_of_work(&self, block: &mut Block) -> String {
        if DIFFICULTY != 0 {
            // If 0, do not apply proof of work
            // initial computed_hash (the initial block.hash)
            let mut computed_hash = String::from(&block.hash);

            // sets the difficulty chars. e.g.: 000 if DIFFICULTY is 3
            let difficulty_chars = "0".repeat(DIFFICULTY); // NOTE: REPEATED

            // check logic
            while !computed_hash.starts_with(difficulty_chars.as_str()) {
                block.nonce += 1; // add 1 to change the hash
                computed_hash = block.compute_hash();
            }

            return computed_hash;
        }

        block.compute_hash()
    }

    /**
     * Inser new transaction to be mined. This transaction is encrypted
     * using the db_access_key. The same key should be provided in order to read
     * the transaction information
     */
    pub fn add_new_transaction(
        &mut self,
        transaction: ContractTransactionData,
        db_access_key: &String,
    ) {
        let encrypted_tx = self.encrypt_transaction(&transaction, db_access_key);
        self.unconfirmed_transactions.push(encrypted_tx);
    }

    pub fn mine(&mut self) -> MineReturnOptions {
        if self.unconfirmed_transactions.is_empty() {
            return MineReturnOptions::BOOL(false);
        }

        let last_block = self.last_block();

        let mut new_block = Block::new(
            last_block.height + 1,
            last_block.hash.clone(),
            None,
            Some(self.unconfirmed_transactions.clone()),
        );

        let proof = self.proof_of_work(&mut new_block);

        self.add_block(&new_block, proof);

        // clear unconfirmed transactions
        self.unconfirmed_transactions.clear();

        return MineReturnOptions::I64(new_block.height);
    }
}
