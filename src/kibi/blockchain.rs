use crate::kibi::block::Block;
use crate::kibi::utils::{hash_generator, DIFFICULTY};

use super::utils::{save_current_block_hash, save_block, load_current_block, load_block, SEARCH_BLOCK_DEPTH};

// A way to inform the kinds of data for a given method
#[derive(Debug)]
pub enum MineReturnOptions {
    BOOL(bool),
    I64(i64),
}

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

    //   println!("LAST BLOCK: {:?}", previous_hash);

      if previous_hash != &block.prev_hash {
          return false;
      }

      if !self.is_valid_proof(&block, proof) {
          return false;
      }

      // NOTE: idk why, but this is failing because what this
      // function receives is a &Block, and the chain is a kind of
      // Vec<Block>. So the compiler is reclaiming about it.

      // So, the way out was recreating a new Block using
      // the block parameter data info
    //   self.chain.push(Block {
    //       index: block.index,
    //       nonce: block.nonce,
    //       transactions: block.transactions.clone(),
    //       timestamp: block.timestamp,
    //       hash: block.hash.clone(),
    //       prev_hash: block.prev_hash.clone()
    //   });

      // Store the current block hash
      save_current_block_hash(block.hash.as_str().as_bytes()).unwrap();

      // Save the current block
      save_block(block).unwrap();

      true
  }

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
    };

    return chain;
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

  pub fn add_new_transaction(&mut self, transaction: String) {
      self.unconfirmed_transactions.push(transaction);
  }

  pub fn mine(&mut self) -> MineReturnOptions {
      if self.unconfirmed_transactions.is_empty() {
          return MineReturnOptions::BOOL(false);
      }

      let last_block = self.last_block();
      println!("LAST BLOCK ------> {:?}", last_block);

      let mut new_block = Block::new(
        last_block.index + 1,
        last_block.hash.clone(),
        None,
        Some(self.unconfirmed_transactions.clone()),
      );

      let proof = self.proof_of_work(&mut new_block);
      self.add_block(&new_block, proof);

      // clear unconfirmed transactions
      self.unconfirmed_transactions.clear();

      return MineReturnOptions::I64(new_block.index);
  }
}