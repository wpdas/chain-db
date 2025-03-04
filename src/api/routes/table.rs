use crate::api::auth::DatabaseAuth;
use crate::api::models::{
    ApiResponse, FindWhereAdvancedRequest, FindWhereRequest, TableData, UpdateTableRequest,
};
use crate::chaindb::ChainDB;
use rocket::serde::json::Json;
use rocket::{get, post};
use std::collections::HashMap;

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

#[post("/table/<table_name>/find", data = "<request>")]
pub fn find_where(
    auth: DatabaseAuth,
    table_name: String,
    request: Json<FindWhereRequest>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    println!("Recebida requisição findWhere para tabela: {}", table_name);
    println!("Critérios: {:?}", request.criteria);
    println!(
        "Limite: {:?}, Reverso: {:?}",
        request.limit, request.reverse
    );

    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => {
                    let reverse = request.reverse.unwrap_or(true);
                    match table.findWhere(request.criteria.clone(), request.limit, reverse) {
                        Ok(records) => {
                            println!("Encontrados {} registros", records.len());
                            let results: Vec<serde_json::Value> =
                                records.into_iter().map(|record| record.to_json()).collect();
                            Json(ApiResponse::success(results))
                        }
                        Err(e) => {
                            println!("Erro ao buscar registros: {}", e);
                            Json(ApiResponse::error(format!("Failed to find records: {}", e)))
                        }
                    }
                }
                Err(e) => {
                    println!("Erro ao criar tabela: {}", e);
                    Json(ApiResponse::error(format!("Failed to create table: {}", e)))
                }
            }
        }
        Err(e) => {
            println!("Erro ao conectar ao banco de dados: {}", e);
            Json(ApiResponse::error(format!(
                "Failed to connect to database: {}",
                e
            )))
        }
    }
}

#[post("/table/<table_name>/find-advanced", data = "<request>")]
pub fn find_where_advanced(
    auth: DatabaseAuth,
    table_name: String,
    request: Json<FindWhereAdvancedRequest>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    println!(
        "Recebida requisição findWhereAdvanced para tabela: {}",
        table_name
    );
    println!("Critérios: {:?}", request.criteria);
    println!(
        "Limite: {:?}, Reverso: {:?}",
        request.limit, request.reverse
    );

    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => {
                    // Converter o formato da requisição para o formato esperado pela função findWhereAdvanced
                    let mut criteria = HashMap::new();
                    for criterion in &request.criteria {
                        criteria.insert(
                            criterion.field.clone(),
                            (criterion.operator.clone(), criterion.value.clone()),
                        );
                    }
                    println!("Critérios convertidos: {:?}", criteria);

                    let reverse = request.reverse.unwrap_or(true);
                    match table.findWhereAdvanced(criteria, request.limit, reverse) {
                        Ok(records) => {
                            println!("Encontrados {} registros", records.len());
                            let results: Vec<serde_json::Value> =
                                records.into_iter().map(|record| record.to_json()).collect();
                            Json(ApiResponse::success(results))
                        }
                        Err(e) => {
                            println!("Erro ao buscar registros: {}", e);
                            Json(ApiResponse::error(format!("Failed to find records: {}", e)))
                        }
                    }
                }
                Err(e) => {
                    println!("Erro ao criar tabela: {}", e);
                    Json(ApiResponse::error(format!("Failed to create table: {}", e)))
                }
            }
        }
        Err(e) => {
            println!("Erro ao conectar ao banco de dados: {}", e);
            Json(ApiResponse::error(format!(
                "Failed to connect to database: {}",
                e
            )))
        }
    }
}
