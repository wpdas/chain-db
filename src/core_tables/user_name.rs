use serde::{Deserialize, Serialize};

pub const USER_NAME_TABLE_NAME: &'static str = "core-user-name";

// Table used to store the user's name when they create an account. This is used
// to make sure no user creates an account with an user_name in use.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserNameTable {
    pub user_name: String,
    pub password_hint: String, // Dica para lembrar da senha
}
