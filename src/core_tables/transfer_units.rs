use serde::{Deserialize, Serialize};

pub const TRANSFER_UNITS_TABLE_NAME: &'static str = "core-transfer-units";

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferUnitsTable {
    pub from: String,
    pub to: String,
    pub units: u64,
}
