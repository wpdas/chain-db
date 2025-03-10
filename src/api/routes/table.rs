use crate::api::auth::DatabaseAuth;
use crate::api::models::{
    ApiResponse, FindWhereAdvancedRequest, FindWhereRequest, PersistTableRequest, TableData,
    UpdateTableRequest,
};
use crate::chaindb::ChainDB;
use rocket::serde::json::Json;
use rocket::{get, post};
use std::collections::HashMap;

#[get("/table/<table_name>")]
pub fn get_table_data(
    auth: DatabaseAuth,
    table_name: &str,
) -> Json<ApiResponse<serde_json::Value>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => match table.get_table() {
                    Ok(data) => {
                        println!("Data before to_json: {:?}", data);
                        let json_data = data.to_json();
                        println!("Data after to_json: {:?}", json_data);
                        Json(ApiResponse::success(json_data))
                    }
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
    table_name: &str,
    request: Json<UpdateTableRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(mut table) => {
                    let data = TableData::from_json(request.data.clone());
                    match table.update(&data, &request.doc_id) {
                        Ok(_) => {
                            // Buscar o registro atualizado pelo doc_id
                            let criteria = HashMap::from([(
                                "doc_id".to_string(),
                                serde_json::Value::String(request.doc_id.clone()),
                            )]);
                            match table.find_where(criteria, Some(1), true) {
                                Ok(records) if !records.is_empty() => {
                                    Json(ApiResponse::success(records[0].to_json()))
                                }
                                Ok(_) => Json(ApiResponse::error(
                                    "Record updated but not found when retrieving".to_string(),
                                )),
                                Err(e) => Json(ApiResponse::error(format!(
                                    "Record updated but failed to retrieve: {}",
                                    e
                                ))),
                            }
                        }
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
    table_name: &str,
    request: Json<PersistTableRequest>,
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
    table_name: &str,
    limit: Option<usize>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let limit = limit.unwrap_or(50);
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => match table.get_history(limit) {
                    Ok(records) => {
                        println!("Records before to_json: {:?}", records);
                        let history: Vec<serde_json::Value> = records
                            .into_iter()
                            .map(|record| {
                                let json = record.to_json();
                                println!("Record after to_json: {:?}", json);
                                json
                            })
                            .collect();
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
    table_name: &str,
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
                    match table.find_where(request.criteria.clone(), request.limit, reverse) {
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
    table_name: &str,
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
                    match table.find_where_advanced(criteria, request.limit, reverse) {
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

/// Lists all tables in the connected database
#[get("/tables")]
pub fn list_tables(auth: DatabaseAuth) -> Json<ApiResponse<Vec<String>>> {
    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.list_tables() {
                Ok(tables) => Json(ApiResponse::success(tables)),
                Err(e) => Json(ApiResponse::error(format!("Failed to list tables: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            "Table not found or wrong Authorization token" // e
        ))),
    }
}

#[get("/table/<table_name>/doc/<doc_id>")]
pub fn get_document_by_id(
    auth: DatabaseAuth,
    table_name: &str,
    doc_id: &str,
) -> Json<ApiResponse<serde_json::Value>> {
    println!(
        "Recebida requisição para buscar documento com doc_id: {} na tabela: {}",
        doc_id, table_name
    );

    match ChainDB::connect(&auth.db_name, &auth.username, &auth.password) {
        Ok(connection) => {
            let db = connection.db;
            match db.create_table::<TableData>(&table_name) {
                Ok(table) => {
                    // Criar critério de busca pelo doc_id
                    let criteria = HashMap::from([(
                        "doc_id".to_string(),
                        serde_json::Value::String(doc_id.to_string()),
                    )]);

                    // Buscar o documento usando a função find_where existente
                    match table.find_where(criteria, Some(1), true) {
                        Ok(records) if !records.is_empty() => {
                            // Retornar o primeiro (e único) registro encontrado
                            Json(ApiResponse::success(records[0].to_json()))
                        }
                        Ok(_) => {
                            // Documento não encontrado
                            Json(ApiResponse::error(format!(
                                "Document with doc_id {} not found",
                                doc_id
                            )))
                        }
                        Err(e) => Json(ApiResponse::error(format!(
                            "Failed to find document: {}",
                            e
                        ))),
                    }
                }
                Err(e) => Json(ApiResponse::error(format!("Failed to create table: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to connect to database: {}",
            "Table not found or wrong Authorization token" // e
        ))),
    }
}
