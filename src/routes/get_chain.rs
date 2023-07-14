use rocket::{get, serde::json::Json};
use serde::Serialize;

use crate::kibi::{block::Block, instance::BlockchainInstance, utils::SEARCH_BLOCK_DEPTH};

#[derive(Serialize)]
pub struct ChainResponse {
    length: usize,
    chain: Vec<Block>,
}

// Return the node's copy of the chain. Endpoint to query all of the data to display

#[get("/")]
pub fn get() -> Json<ChainResponse> {
    let chain = BlockchainInstance::chain(SEARCH_BLOCK_DEPTH);

    let response = ChainResponse {
        length: chain.len(),
        chain: chain,
    };

    Json(response)
}
