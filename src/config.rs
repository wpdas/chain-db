use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub user: String,
    pub password: String,
}

impl Config {
    pub fn new(name: &str, user: &str, password: &str) -> Self {
        Self {
            name: name.to_string(),
            user: user.to_string(),
            password: password.to_string(),
        }
    }
}
