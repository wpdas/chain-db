use crate::api::auth::DatabaseAuth;
use crate::api::models::{ApiResponse, TableData, UpdateTableRequest};
use crate::chaindb::ChainDB;
use rocket::serde::json::Json;
use rocket::{get, post};

#[get("/table/<table_name>")]
pub fn get_table_data(
    auth: DatabaseAuth,
    table_name: String,
) -> Json<ApiResponse<serde_json::Value>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => match table.get_table() {
                    Ok(data) => Json(ApiResponse::success(data.to_json())),
                    Err(e) => Json(ApiResponse::error(format!("Failed to get data: {}", e))),
                },
                Err(e) => Json(ApiResponse::error(format!("Failed to create table: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            e
        ))),
    }
}

#[post("/table/<table_name>/update", data = "<request>")]
pub fn update_table(
    auth: DatabaseAuth,
    table_name: String,
    request: Json<UpdateTableRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(mut table) => {
                    let data = TableData::from_json(request.data.clone());
                    match table.update(&data) {
                        Ok(_) => match table.get_table() {
                            Ok(latest) => Json(ApiResponse::success(latest.to_json())),
                            Err(e) => Json(ApiResponse::error(format!(
                                "Failed to get latest data: {}",
                                e
                            ))),
                        },
                        Err(e) => Json(ApiResponse::error(format!("Failed to update data: {}", e))),
                    }
                }
                Err(e) => Json(ApiResponse::error(format!("Failed to create table: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            e
        ))),
    }
}

#[post("/table/<table_name>/persist", data = "<request>")]
pub fn persist_table(
    auth: DatabaseAuth,
    table_name: String,
    request: Json<UpdateTableRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(mut table) => {
                    let data = TableData::from_json(request.data.clone());
                    match table.persist(&data) {
                        Ok(_) => match table.get_table() {
                            Ok(latest) => Json(ApiResponse::success(latest.to_json())),
                            Err(e) => Json(ApiResponse::error(format!(
                                "Failed to get latest data: {}",
                                e
                            ))),
                        },
                        Err(e) => {
                            Json(ApiResponse::error(format!("Failed to persist data: {}", e)))
                        }
                    }
                }
                Err(e) => Json(ApiResponse::error(format!("Failed to create table: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            e
        ))),
    }
}

#[get("/table/<table_name>/history?<limit>")]
pub fn get_history(
    auth: DatabaseAuth,
    table_name: String,
    limit: Option<usize>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let limit = limit.unwrap_or(50);
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => match table.get_history(limit) {
                    Ok(records) => {
                        let history: Vec<serde_json::Value> =
                            records.into_iter().map(|record| record.to_json()).collect();
                        Json(ApiResponse::success(history))
                    }
                    Err(e) => Json(ApiResponse::error(format!("Failed to get history: {}", e))),
                },
                Err(e) => Json(ApiResponse::error(format!("Failed to create table: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            "Table not found or wrong Authorization token" // e
        ))),
    }
}
