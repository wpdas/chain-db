use crate::table::ComparisonOperator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableData {
    pub data: HashMap<String, serde_json::Value>,
}

impl TableData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Object(map) => Self {
                data: map.into_iter().collect(),
            },
            _ => Self::new(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::Value::Object(self.data.clone().into_iter().collect())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectDatabaseRequest {
    pub name: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTableRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetHistoryRequest {
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindWhereRequest {
    pub criteria: HashMap<String, serde_json::Value>,
    pub limit: Option<usize>,
    pub reverse: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindWhereAdvancedCriteria {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindWhereAdvancedRequest {
    pub criteria: Vec<FindWhereAdvancedCriteria>,
    pub limit: Option<usize>,
    pub reverse: Option<bool>,
}
