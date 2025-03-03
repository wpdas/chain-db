use crate::api::models::{ApiResponse, ConnectDatabaseRequest, CreateDatabaseRequest};
use crate::chaindb::ChainDB;
use rocket::post;
use rocket::serde::json::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub name: String,
    pub user: String,
    pub old_password: String,
    pub new_password: String,
}

#[post("/database/create", data = "<request>")]
pub fn create_database(request: Json<CreateDatabaseRequest>) -> Json<ApiResponse<String>> {
    match ChainDB::create_database(&request.name, &request.user, &request.password) {
        Ok(_) => Json(ApiResponse::success(
            "Database created successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to create database: {}",
            e
        ))),
    }
}

#[post("/database/connect", data = "<request>")]
pub fn connect_database(request: Json<ConnectDatabaseRequest>) -> Json<ApiResponse<String>> {
    match ChainDB::connect(&request.name, &request.user, &request.password) {
        Ok(connection) => Json(ApiResponse::success(connection.auth_token)),
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            "Table not found or wrong user/password" // e // "Table not found or wrong user/password"
        ))),
    }
}

#[post("/database/change-password", data = "<request>")]
pub fn change_password(request: Json<ChangePasswordRequest>) -> Json<ApiResponse<String>> {
    match ChainDB::connect(&request.name, &request.user, &request.old_password) {
        Ok(connection) => {
            let mut db = connection.db;
            match db.change_password(&request.new_password) {
                Ok(_) => Json(ApiResponse::success(
                    "Password changed successfully".to_string(),
                )),
                Err(e) => Json(ApiResponse::error(format!(
                    "Failed to change password: {}",
                    e
                ))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            "Table not found or wrong user/password" // e // "Table not found"
        ))),
    }
}
