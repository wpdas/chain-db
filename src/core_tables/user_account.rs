use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub const USER_ACCOUNT_TABLE_NAME: &'static str = "core-user-account";

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct UserAccountTable {
    pub id: String, // Used to refer the user
    pub user_name: String,
    pub units: u64,
}
