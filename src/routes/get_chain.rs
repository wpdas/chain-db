use rocket::{get, serde::json::Json};
use serde::Serialize;

use crate::kibi::{
  instance::BlockchainInstance,
  block::BlockJson, utils::{block_to_blockjson, SEARCH_BLOCK_DEPTH}
};

#[derive(Serialize)]
pub struct ChainResponse {
  length: usize,
  chain: Vec<BlockJson>,
}

// Return the node's copy of the chain. Endpoint to query all of the data to display

#[get("/")]
pub fn get() -> Json<ChainResponse> {
  let mut chain_data: Vec<BlockJson> = vec![];

  let chain = BlockchainInstance::blockchain().chain(SEARCH_BLOCK_DEPTH);

  for block in chain {

    // decode transactions
    let block_json = block_to_blockjson(&block);

    // push the current block to the list of blocks
    chain_data.push(block_json)
  }

  let response = ChainResponse {
    length: chain_data.len(),
    chain: chain_data,
  };

  Json(response)
}