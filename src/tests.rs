#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use crate::api::models::TableData;
    use crate::chaindb::ChainDB;

    #[test]
    fn test_db_creation() {
        let result = ChainDB::create_database("test_db", "test_user", "test_pass");
        assert!(result.is_ok());

        // Tenta conectar ao banco criado
        let connection = ChainDB::connect("test_db", "test_user", "test_pass");
        assert!(connection.is_ok());

        // Limpa os arquivos de teste
        std::fs::remove_dir_all(".chain_db_data/test_db").unwrap();
    }

    #[test]
    fn test_table_operations() {
        thread::sleep(Duration::from_secs(2)); // Wait for 2 seconds to ensure the database folder was cleaned

        // Cria e conecta ao banco
        let _ = ChainDB::create_database("test_db", "test_user", "test_pass");
        let connection = ChainDB::connect("test_db", "test_user", "test_pass").unwrap();

        // Cria uma tabela
        let mut table = connection
            .db
            .create_table::<TableData>("test_table")
            .unwrap();

        // Testa persist
        let mut table_data = TableData::new();
        table_data
            .data
            .insert("test_key".to_string(), serde_json::json!("test_value"));
        assert!(table.persist(&table_data).is_ok());

        table_data
            .data
            .insert("test_key".to_string(), serde_json::json!("test_value_2"));
        assert!(table.persist(&table_data).is_ok());

        // Testa get_table (último registro)
        let result = table.get_table().unwrap();
        assert_eq!(result.data.get("test_key").unwrap(), "test_value_2");

        // Testa update
        // Primeiro obtém o doc_id do registro que queremos atualizar
        let doc_id = table.get_table().unwrap().doc_id.unwrap();

        let mut new_data = TableData::new();
        new_data
            .data
            .insert("test_key".to_string(), serde_json::json!("updated_value"));
        assert!(table.update(&new_data, &doc_id).is_ok());

        // Testa get_history
        let history = table.get_history(50).unwrap();
        println!("history: {:?}", history);
        assert_eq!(history.len(), 2);
        // O registro mais recente (updated_value) deve estar no índice 0
        assert_eq!(history[0].data.get("test_key").unwrap(), "updated_value");
        // O registro mais antigo (test_value) deve estar no índice 1
        assert_eq!(history[1].data.get("test_key").unwrap(), "test_value");

        // Limpa os arquivos de teste
        std::fs::remove_dir_all(".chain_db_data/test_db").unwrap();
    }
}
