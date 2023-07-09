use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
struct AccountData {
  accounts: HashMap<String, String>,
}

pub struct AccountId {}
impl AccountId {
  /**
   * Generates a hash for this user
   */
  pub fn parse(account: String) -> String {
    // TODO: usar key pair (public / private key)
    sha256::digest(account)
  }
}