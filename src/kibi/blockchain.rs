use borsh::BorshDeserialize;

use crate::kibi::block::Block;
use crate::kibi::utils::{hash_generator, DIFFICULTY};

use super::encryption::AesEcb;
use super::types::{ContractTransactionData, ContractTransactionDataJson};
use super::utils::{save_current_block_hash, save_block, load_current_block, load_block, SEARCH_BLOCK_DEPTH, block_to_blockjson};

// A way to inform the kinds of data for a given method
#[derive(Debug)]
pub enum MineReturnOptions {
    BOOL(bool),
    I64(i64),
}

#[derive(Clone)]
pub struct Blockchain {
//   pub chain: Vec<Block>, // Talvez vai ser removido (os arquivos vivem localmente)
  pub unconfirmed_transactions: Vec<String>
}

impl Blockchain {
  // Auto create an instance of Blockchain and init it
  pub fn new() -> Blockchain {
    let mut instance = Blockchain {
        // chain: Vec::new(),
        unconfirmed_transactions: Vec::new()
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
    // TODO: the block decription should occur here

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
    };

    return chain;
  }

  /**
   * Run over blocks to get the transactions under a specific contract
   * Gets all data/transactions inside the contract(living inside blocks) and try
   * to decrypt it using the given db_access_key, if it succeed, return the transaction
   * 
   * Preffer to use `get_last_transaction_data_under_contract` that's going to return
   * only one transaction/data (the most recent one)
   */
  pub fn get_transactions_under_contract(&self, contract_id: String, depth: u64) -> Vec<ContractTransactionDataJson> {
    // TODO: the block decription should occur here;
    let mut transactions: Vec<ContractTransactionDataJson> = vec![];
    let current_block = load_current_block().unwrap();
    let mut prev_block_hash = current_block.prev_hash.to_owned();
    
    let mut n = 0;

    while n < depth {
        let block_opt = load_block(prev_block_hash.to_owned());

        if block_opt.is_none() {
            break;
        }

        let block = block_opt.unwrap();

        // decode transactions
        let block_json = block_to_blockjson(&block);
        for tx in block_json.transactions {
            if tx["contract_id"].is_string() && tx["contract_id"] == contract_id {

                let dec_tx = serde_json::from_value::<ContractTransactionData>(tx).unwrap();

                // create json version from dec_tx
                let dec_tx_json = ContractTransactionDataJson {
                    tx_type: dec_tx.tx_type,
                    contract_id: dec_tx.contract_id,
                    timestamp: dec_tx.timestamp,
                    data: serde_json::from_str(&dec_tx.data).unwrap(),
                    block_hash: block_json.hash.clone(),
                    block_height: block_json.height,
                };

                transactions.push(dec_tx_json);
            }
          }

        // set the new prev_block_hash
        prev_block_hash = block.prev_hash;

        n += 1;
    };

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
  pub fn get_last_transaction_data_under_contract(&self, contract_id: String, db_access_key: &String, depth: u64) -> Option<ContractTransactionDataJson> {
    // TODO: the block decription should occur here;
    let current_block = load_current_block().unwrap();
    let mut prev_block_hash = current_block.prev_hash.to_owned();
    let mut n = 0;

    while n < depth {
        let block_opt = load_block(prev_block_hash.to_owned());

        if block_opt.is_none() {
            break;
        }

        let block = block_opt.unwrap();

        // decode transactions
        // let block_json = block_to_blockjson(&block);
        for encrypted_tx in block.transactions {
            // Try to decrypt transaction using the given db_access_key
            let tx_opt = AesEcb::decode(&encrypted_tx, &db_access_key);
            if tx_opt.is_none() {
                return None;
            }
            // Try decode using Borsh
            let tx = ContractTransactionData::try_from_slice(
                tx_opt.unwrap().as_bytes()
            ).unwrap();
            
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

        // set the new prev_block_hash
        prev_block_hash = block.prev_hash;

        n += 1;
    };

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
    if DIFFICULTY != 0 { // If 0, do not apply proof of work
        // initial computed_hash (the initial block.hash)
        let mut computed_hash = String::from(&block.hash);

        // sets the difficulty chars. e.g.: 000 if DIFFICULTY is 3
        let difficulty_chars = "0".repeat(DIFFICULTY); // NOTE: REPEATED

        // check logic
        while !computed_hash.starts_with(difficulty_chars.as_str()) {
            block.nonce += 1; // add 1 to change the hash
            computed_hash = block.compute_hash();
        }

        return computed_hash
    }

    block.compute_hash()
  }

  /**
   * Inser new transaction to be mined. This transaction is encrypted
   * using the db_access_key. The same key should be provided in order to read
   * the transaction information
   */
  pub fn add_new_transaction(&mut self, transaction: String) {
    self.unconfirmed_transactions.push(transaction);
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