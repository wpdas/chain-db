use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use uuid::Uuid;

use crate::encryption::DataEncryption;
use crate::errors::ChainDBError;
use crate::events::{emit_table_persist, emit_table_update};

const RECORDS_PER_FILE: usize = 1000;
pub const METADATA_FILE: &str = "metadata.cdb";

/// Operadores de comparação disponíveis para consultas avançadas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Igual a (==)
    Eq,
    /// Diferente de (!=)
    Ne,
    /// Maior que (>)
    Gt,
    /// Maior ou igual a (>=)
    Ge,
    /// Menor que (<)
    Lt,
    /// Menor ou igual a (<=)
    Le,
    /// Contém (para strings e arrays)
    Contains,
    /// Começa com (para strings)
    StartsWith,
    /// Termina com (para strings)
    EndsWith,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableMetadata {
    pub current_file: u32,
    pub total_records: u64,
}

impl TableMetadata {
    fn new() -> Self {
        Self {
            current_file: 0,
            total_records: 0,
        }
    }
}

#[derive(Debug)]
pub struct Table<T> {
    path: PathBuf,
    metadata: TableMetadata,
    encryption: DataEncryption,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Table<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(path: PathBuf, encryption: DataEncryption) -> Result<Self, ChainDBError> {
        fs::create_dir_all(&path)?;
        let metadata_path = path.join(METADATA_FILE);

        let metadata = if metadata_path.exists() {
            let encrypted_metadata = fs::read(&metadata_path)?;
            let metadata_json = encryption.decrypt(&encrypted_metadata)?;
            serde_json::from_slice(&metadata_json)?
        } else {
            let metadata = TableMetadata::new();
            let metadata_json = serde_json::to_vec(&metadata)?;
            let encrypted_metadata = encryption.encrypt(&metadata_json)?;
            let mut file = File::create(&metadata_path)?;
            file.write_all(&encrypted_metadata)?;
            metadata
        };

        Ok(Self {
            path,
            metadata,
            encryption,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn persist(&mut self, record: &T) -> Result<(), ChainDBError> {
        // Converter o registro para Value para adicionar o doc_id
        let mut record_value = serde_json::to_value(record)?;
        println!("Record before adding doc_id: {:?}", record_value);

        // Remover qualquer doc_id que o usuário tenha tentado incluir
        if let serde_json::Value::Object(ref mut map) = record_value {
            // Se houver um campo data que é um objeto, remover doc_id dele também
            if let Some(serde_json::Value::Object(ref mut data_map)) = map.get_mut("data") {
                data_map.remove("doc_id");
            }

            // Remover doc_id do objeto raiz
            map.remove("doc_id");

            // Gerar um UUID v4 único para o documento
            let doc_id = Uuid::new_v4().to_string();

            // Adicionar o doc_id gerado pelo sistema
            map.insert(
                "doc_id".to_string(),
                serde_json::Value::String(doc_id.clone()),
            );
        }

        println!("Record after adding doc_id: {:?}", record_value);

        // Converter de volta para bytes
        let record_json = serde_json::to_vec(&record_value)?;
        let encrypted_record = self.encryption.encrypt(&record_json)?;

        // Calculate current file based on total records
        let current_file = (self.metadata.total_records as usize / RECORDS_PER_FILE) as u32;
        let file_name = format!("data_{}.cdb", current_file);
        let file_path = self.path.join(&file_name);

        // Create or append to file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        // Convert encrypted data to base64
        let base64_str = general_purpose::STANDARD.encode(&encrypted_record);
        writeln!(file, "{}", base64_str)?;

        // Update metadata
        self.metadata.current_file = current_file;
        self.metadata.total_records += 1;

        // Save metadata
        let metadata_json = serde_json::to_vec(&self.metadata)?;
        let encrypted_metadata = self.encryption.encrypt(&metadata_json)?;
        let metadata_path = self.path.join(METADATA_FILE);
        fs::write(metadata_path, encrypted_metadata)?;

        // Emitir evento de persistência
        if let Ok(record_value) = serde_json::to_value(record) {
            // Extrair o nome do banco de dados do caminho
            let db_name = self
                .path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Extrair o nome da tabela do caminho
            let table_name = self
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Extrair apenas os dados internos do campo 'data' se existir
            let data_to_emit = if let Some(data_obj) = record_value.get("data") {
                Some(data_obj.clone())
            } else {
                Some(record_value)
            };

            emit_table_persist(db_name, table_name, data_to_emit);
        }

        Ok(())
    }

    pub fn update(&mut self, record: &T, doc_id: &str) -> Result<(), ChainDBError> {
        if self.metadata.total_records == 0 {
            return Err(ChainDBError::RecordNotFound(
                "No records exist in this table".to_string(),
            ));
        }

        // Procura o registro específico pelo doc_id
        let mut found = false;

        // Iterar por todos os arquivos de dados, do mais recente para o mais antigo
        for file_index in (0..=self.metadata.current_file).rev() {
            let file_name = format!("data_{}.cdb", file_index);
            let file_path = self.path.join(&file_name);

            if !file_path.exists() {
                continue;
            }

            // Ler o arquivo linha por linha
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            let mut updated_lines = Vec::new();
            let mut file_modified = false;

            for line in reader.lines() {
                let line = line?;
                let decoded = general_purpose::STANDARD.decode(&line)?;
                let decrypted = self.encryption.decrypt(&decoded)?;
                let record_value: serde_json::Value = serde_json::from_slice(&decrypted)?;

                // Verificar se este é o registro que estamos procurando
                if let Some(record_doc_id) = record_value.get("doc_id") {
                    if let Some(id_str) = record_doc_id.as_str() {
                        if id_str == doc_id {
                            // Encontramos o registro, vamos atualizá-lo
                            // Converter o registro para Value para preservar o doc_id
                            let mut updated_value = serde_json::to_value(record)?;

                            // Remover qualquer doc_id que o usuário tenha tentado incluir
                            if let serde_json::Value::Object(ref mut map) = &mut updated_value {
                                // Se houver um campo data que é um objeto, remover doc_id dele também
                                if let Some(serde_json::Value::Object(ref mut data_map)) =
                                    map.get_mut("data")
                                {
                                    data_map.remove("doc_id");
                                }

                                // Remover doc_id do objeto raiz
                                map.remove("doc_id");

                                // Preservar o doc_id original
                                map.insert(
                                    "doc_id".to_string(),
                                    serde_json::Value::String(doc_id.to_string()),
                                );
                            }

                            // Criptografar e adicionar à lista de linhas atualizadas
                            let updated_json = serde_json::to_vec(&updated_value)?;
                            let encrypted_updated = self.encryption.encrypt(&updated_json)?;
                            let base64_str = general_purpose::STANDARD.encode(&encrypted_updated);
                            updated_lines.push(base64_str);

                            found = true;
                            file_modified = true;
                            continue;
                        }
                    }
                }

                // Se não for o registro que estamos procurando, manter como está
                updated_lines.push(line);
            }

            // Se o arquivo foi modificado, escrever as alterações de volta
            if file_modified {
                let mut file = fs::File::create(&file_path)?;
                for line in updated_lines {
                    writeln!(file, "{}", line)?;
                }
                break;
            }
        }

        // Se não encontramos o registro, retornar um erro
        if !found {
            return Err(ChainDBError::RecordNotFound(format!(
                "Record with doc_id {} not found",
                doc_id
            )));
        }

        // Emitir evento de atualização
        if let Ok(record_value) = serde_json::to_value(record) {
            // Extrair o nome do banco de dados do caminho
            let db_name = self
                .path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Extrair o nome da tabela do caminho
            let table_name = self
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Extrair apenas os dados internos do campo 'data' se existir
            let data_to_emit = if let Some(data_obj) = record_value.get("data") {
                Some(data_obj.clone())
            } else {
                Some(record_value)
            };

            emit_table_update(db_name, table_name, data_to_emit);
        }

        Ok(())
    }

    /// Recupera um registro específico pelo índice.
    ///
    /// # Argumentos
    ///
    /// * `index` - O índice do registro a ser recuperado (baseado em 0)
    ///
    /// # Retornos
    ///
    /// * `Ok(T)` - O registro encontrado no índice especificado
    /// * `Err(ChainDBError)` - Se o índice for inválido ou ocorrer um erro durante a leitura
    ///
    pub fn get(&self, index: u64) -> Result<T, ChainDBError> {
        if index >= self.metadata.total_records {
            return Err(ChainDBError::SerializationError(
                "Record not found".to_string(),
            ));
        }

        let file_index = index as usize / RECORDS_PER_FILE;
        let record_index = index as usize % RECORDS_PER_FILE;

        let file_name = format!("data_{}.cdb", file_index);
        let file_path = self.path.join(&file_name);

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (i, line) in reader.lines().enumerate() {
            if i == record_index {
                let line = line?;
                let encrypted_data = general_purpose::STANDARD
                    .decode(line)
                    .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;
                let decrypted_data = self.encryption.decrypt(&encrypted_data)?;
                return Ok(serde_json::from_slice(&decrypted_data)?);
            }
        }

        Err(ChainDBError::SerializationError(
            "Record not found".to_string(),
        ))
    }

    /// Recupera os N registros mais recentes em ordem cronológica reversa.
    ///
    /// # Argumentos
    ///
    /// * `limit` - O número máximo de registros a serem retornados
    ///
    /// # Retornos
    ///
    /// * `Ok(Vec<T>)` - Um vetor contendo até `limit` registros, ordenados do mais recente para o mais antigo
    /// * `Err(ChainDBError)` - Se ocorrer um erro durante a leitura dos registros
    ///
    /// # Notas
    ///
    /// * Se o número total de registros for menor que o limite, retorna todos os registros disponíveis
    /// * Se não houver registros, retorna um vetor vazio
    /// * Os registros são lidos de múltiplos arquivos se necessário
    pub fn get_history(&self, limit: usize) -> Result<Vec<T>, ChainDBError> {
        if self.metadata.total_records == 0 {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();
        let mut remaining = limit.min(self.metadata.total_records as usize);
        let mut current_file = self.metadata.current_file;

        while remaining > 0 && current_file <= self.metadata.current_file {
            let file_name = format!("data_{}.cdb", current_file);
            let file_path = self.path.join(&file_name);

            if !file_path.exists() {
                current_file = current_file.wrapping_sub(1);
                continue;
            }

            let file = File::open(&file_path)?;
            let mut reader = BufReader::new(file);
            let mut lines = Vec::with_capacity(remaining);

            // Lê as linhas do final do arquivo
            reader.seek(SeekFrom::End(0))?;
            let mut position = reader.stream_position()?;
            let mut line = String::new();

            while position > 0 && remaining > 0 {
                let mut found_newline = false;
                while position > 0 && !found_newline {
                    position -= 1;
                    reader.seek(SeekFrom::Start(position))?;
                    let byte = &mut [0];
                    reader.read_exact(byte)?;
                    found_newline = byte[0] == b'\n';
                }

                if position == 0 {
                    // Estamos no início do arquivo
                    reader.seek(SeekFrom::Start(0))?;
                } else {
                    // Volta uma posição antes do \n para ler a linha completa
                    reader.seek(SeekFrom::Start(position + 1))?;
                }

                // Lê a linha atual
                line.clear();
                reader.read_line(&mut line)?;

                // Remove caracteres de controle do final
                if line.ends_with('\n') {
                    line.pop();
                }
                if line.ends_with('\r') {
                    line.pop();
                }

                if !line.is_empty() {
                    lines.push(line.clone());
                    remaining -= 1;
                }

                if position > 0 {
                    position -= 1;
                }
            }

            // Processa as linhas
            for line in lines.iter() {
                let encrypted_data = general_purpose::STANDARD
                    .decode(line)
                    .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;
                let decrypted_data = self.encryption.decrypt(&encrypted_data)?;
                let record: T = serde_json::from_slice(&decrypted_data)?;
                records.push(record);
            }

            current_file = current_file.wrapping_sub(1);
        }

        Ok(records)
    }

    pub fn get_table(&self) -> Result<T, ChainDBError> {
        if self.metadata.total_records == 0 {
            return Err(ChainDBError::SerializationError(
                "No records found".to_string(),
            ));
        }

        let file_name = format!("data_{}.cdb", self.metadata.current_file);
        let file_path = self.path.join(&file_name);

        let content = fs::read_to_string(&file_path)?;
        let last_line = content
            .lines()
            .last()
            .ok_or_else(|| ChainDBError::SerializationError("No records found".to_string()))?;

        let encrypted_data = general_purpose::STANDARD
            .decode(last_line)
            .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;
        let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

        // Deserializar os dados diretamente, sem modificações
        // O doc_id já está incluído nos dados serializados
        Ok(serde_json::from_slice(&decrypted_data)?)
    }

    /// Busca registros que correspondem a critérios específicos.
    ///
    /// # Argumentos
    ///
    /// * `criteria` - Um HashMap contendo os critérios de busca, onde as chaves são os nomes dos campos
    ///   e os valores são os valores esperados para esses campos.
    /// * `limit` - Número máximo de registros a serem retornados (opcional). Se não for especificado,
    ///   retorna todos os registros que correspondem aos critérios.
    /// * `reverse` - Se verdadeiro, busca do registro mais recente para o mais antigo (padrão: true).
    ///
    /// # Retornos
    ///
    /// * `Ok(Vec<T>)` - Um vetor contendo os registros que correspondem aos critérios
    /// * `Err(ChainDBError)` - Se ocorrer um erro durante a busca
    ///
    pub fn find_where(
        &self,
        criteria: HashMap<String, serde_json::Value>,
        limit: Option<usize>,
        reverse: bool,
    ) -> Result<Vec<T>, ChainDBError> {
        // # Exemplo
        //
        // ```
        // let mut criteria = HashMap::new();
        // criteria.insert("nome".to_string(), serde_json::json!("João"));
        // criteria.insert("idade".to_string(), serde_json::json!(30));
        // let resultados = tabela.findWhere(criteria, Some(10), true)?;
        // ```

        if self.metadata.total_records == 0 {
            return Ok(Vec::new());
        }

        println!("Iniciando busca findWhere com critérios: {:?}", criteria);
        println!("Limite: {:?}, Ordem reversa: {}", limit, reverse);

        let mut matching_records = Vec::new();
        let max_records = limit.unwrap_or(usize::MAX);

        if reverse {
            // Busca do mais recente para o mais antigo
            let mut current_file = self.metadata.current_file;

            while matching_records.len() < max_records {
                let file_name = format!("data_{}.cdb", current_file);
                let file_path = self.path.join(&file_name);

                if !file_path.exists() {
                    if current_file == 0 {
                        break;
                    }
                    current_file = current_file.wrapping_sub(1);
                    continue;
                }

                let file = File::open(&file_path)?;
                let reader = BufReader::new(file);

                // Lê todas as linhas do arquivo
                let lines: Result<Vec<String>, std::io::Error> = reader.lines().collect();
                let lines = lines?;

                // Processa as linhas em ordem reversa
                for line in lines.iter().rev() {
                    if matching_records.len() >= max_records {
                        break;
                    }

                    let encrypted_data = general_purpose::STANDARD
                        .decode(line)
                        .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;
                    let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

                    // Converte o registro para Value para facilitar a comparação
                    let record_value: serde_json::Value = serde_json::from_slice(&decrypted_data)?;

                    println!("record_value: {:?}", record_value);

                    // Verifica se o registro corresponde a todos os critérios
                    if matches_criteria(&record_value, &criteria) {
                        let record: T = serde_json::from_slice(&decrypted_data)?;
                        matching_records.push(record);
                        println!("record: {:?}", decrypted_data);
                    }
                }

                if current_file == 0 {
                    break;
                }
                current_file = current_file.wrapping_sub(1);
            }
        } else {
            // Busca do mais antigo para o mais recente (código original)
            for file_index in 0..=self.metadata.current_file {
                if matching_records.len() >= max_records {
                    break;
                }

                let file_name = format!("data_{}.cdb", file_index);
                let file_path = self.path.join(&file_name);

                if !file_path.exists() {
                    continue;
                }

                let file = File::open(&file_path)?;
                let reader = BufReader::new(file);

                // Processa cada linha do arquivo
                for line in reader.lines() {
                    if matching_records.len() >= max_records {
                        break;
                    }

                    let line = line?;
                    let encrypted_data = general_purpose::STANDARD
                        .decode(line)
                        .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;
                    let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

                    // Converte o registro para Value para facilitar a comparação
                    let record_value: serde_json::Value = serde_json::from_slice(&decrypted_data)?;

                    // Verifica se o registro corresponde a todos os critérios
                    if matches_criteria(&record_value, &criteria) {
                        let record: T = serde_json::from_slice(&decrypted_data)?;
                        matching_records.push(record);
                    }
                }
            }
        }

        Ok(matching_records)
    }

    /// Busca avançada de registros com suporte a diferentes operadores de comparação.
    ///
    /// # Argumentos
    ///
    /// * `criteria` - Um HashMap contendo os critérios de busca, onde as chaves são os nomes dos campos
    ///   e os valores são tuplas contendo o operador de comparação e o valor esperado.
    /// * `limit` - Número máximo de registros a serem retornados (opcional).
    /// * `reverse` - Se verdadeiro, busca do registro mais recente para o mais antigo (padrão: true).
    ///
    /// # Retornos
    ///
    /// * `Ok(Vec<T>)` - Um vetor contendo os registros que correspondem aos critérios
    /// * `Err(ChainDBError)` - Se ocorrer um erro durante a busca
    ///
    pub fn find_where_advanced(
        &self,
        criteria: HashMap<String, (ComparisonOperator, serde_json::Value)>,
        limit: Option<usize>,
        reverse: bool,
    ) -> Result<Vec<T>, ChainDBError> {
        // # Exemplo
        //
        // ```
        // let mut criteria = HashMap::new();
        // criteria.insert("idade".to_string(), (ComparisonOperator::Gt, serde_json::json!(30)));
        // criteria.insert("nome".to_string(), (ComparisonOperator::Contains, serde_json::json!("Silva")));
        // let resultados = tabela.findWhereAdvanced(criteria, Some(10), true)?;
        // ```

        if self.metadata.total_records == 0 {
            return Ok(Vec::new());
        }

        println!("Iniciando busca findWhereAdvanced");
        for (field, (op, value)) in &criteria {
            println!("Critério: campo '{}' {:?} valor '{:?}'", field, op, value);
        }
        println!("Limite: {:?}, Ordem reversa: {}", limit, reverse);

        let mut matching_records = Vec::new();
        let max_records = limit.unwrap_or(usize::MAX);

        let process_record =
            |line: &str, matching_records: &mut Vec<T>| -> Result<(), ChainDBError> {
                let encrypted_data = general_purpose::STANDARD
                    .decode(line)
                    .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;
                let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

                // Converte o registro para Value para facilitar a comparação
                let record_value: serde_json::Value = serde_json::from_slice(&decrypted_data)?;

                // Verifica se o registro corresponde a todos os critérios
                if matches_criteria_advanced(&record_value, &criteria) {
                    let record: T = serde_json::from_slice(&decrypted_data)?;
                    matching_records.push(record);
                }
                Ok(())
            };

        if reverse {
            // Busca do mais recente para o mais antigo
            let mut current_file = self.metadata.current_file;

            while matching_records.len() < max_records {
                let file_name = format!("data_{}.cdb", current_file);
                let file_path = self.path.join(&file_name);

                if !file_path.exists() {
                    if current_file == 0 {
                        break;
                    }
                    current_file = current_file.wrapping_sub(1);
                    continue;
                }

                let file = File::open(&file_path)?;
                let reader = BufReader::new(file);

                // Lê todas as linhas do arquivo
                let lines: Result<Vec<String>, std::io::Error> = reader.lines().collect();
                let lines = lines?;

                // Processa as linhas em ordem reversa
                for line in lines.iter().rev() {
                    if matching_records.len() >= max_records {
                        break;
                    }
                    process_record(line, &mut matching_records)?;
                }

                if current_file == 0 {
                    break;
                }
                current_file = current_file.wrapping_sub(1);
            }
        } else {
            // Busca do mais antigo para o mais recente
            for file_index in 0..=self.metadata.current_file {
                if matching_records.len() >= max_records {
                    break;
                }

                let file_name = format!("data_{}.cdb", file_index);
                let file_path = self.path.join(&file_name);

                if !file_path.exists() {
                    continue;
                }

                let file = File::open(&file_path)?;
                let reader = BufReader::new(file);

                // Processa cada linha do arquivo
                for line_result in reader.lines() {
                    if matching_records.len() >= max_records {
                        break;
                    }
                    let line = line_result?;
                    process_record(&line, &mut matching_records)?;
                }
            }
        }

        Ok(matching_records)
    }
}

/// Função auxiliar para verificar se um registro corresponde aos critérios especificados
fn matches_criteria(
    record: &serde_json::Value,
    criteria: &HashMap<String, serde_json::Value>,
) -> bool {
    if let serde_json::Value::Object(record_obj) = record {
        // Verifica cada critério
        for (field, expected_value) in criteria {
            // Primeiro verifica se o campo existe no objeto raiz (para doc_id)
            if let Some(actual_value) = record_obj.get(field) {
                // Se o valor do campo não corresponde ao critério, retorna false
                if actual_value != expected_value {
                    println!(
                        "Campo '{}' no objeto raiz não corresponde. Esperado: {:?}, Atual: {:?}",
                        field, expected_value, actual_value
                    );
                    return false;
                }
                // Se encontrou e corresponde, continua para o próximo critério
                continue;
            }

            // Se não encontrou no objeto raiz, verifica no objeto data
            if let Some(serde_json::Value::Object(data_obj)) = record_obj.get("data") {
                match data_obj.get(field) {
                    Some(actual_value) => {
                        // Se o valor do campo não corresponde ao critério, retorna false
                        if actual_value != expected_value {
                            println!(
                                "Campo '{}' no objeto data não corresponde. Esperado: {:?}, Atual: {:?}",
                                field, expected_value, actual_value
                            );
                            return false;
                        }
                    }
                    None => {
                        // Se o campo não existe no registro, retorna false
                        println!("Campo '{}' não encontrado no registro", field);
                        return false;
                    }
                }
            } else {
                // O registro não tem um campo "data" válido e o campo não foi encontrado no objeto raiz
                println!(
                    "Campo '{}' não encontrado no registro e não há objeto 'data' válido",
                    field
                );
                return false;
            }
        }
        // Todos os critérios foram atendidos
        println!("Registro corresponde a todos os critérios");
        true
    } else {
        // O registro não é um objeto JSON
        println!("Registro não é um objeto JSON: {:?}", record);
        false
    }
}

/// Função auxiliar para verificar se um registro corresponde aos critérios avançados especificados
fn matches_criteria_advanced(
    record: &serde_json::Value,
    criteria: &HashMap<String, (ComparisonOperator, serde_json::Value)>,
) -> bool {
    if let serde_json::Value::Object(record_obj) = record {
        // Verifica cada critério
        for (field, (operator, expected_value)) in criteria {
            // Primeiro verifica se o campo existe no objeto raiz (para doc_id)
            if let Some(actual_value) = record_obj.get(field) {
                // Verifica se o valor do campo corresponde ao critério com o operador especificado
                if !compare_values(actual_value, expected_value, operator) {
                    println!("Campo '{}' no objeto raiz não corresponde com operador {:?}. Esperado: {:?}, Atual: {:?}", 
                        field, operator, expected_value, actual_value);
                    return false;
                }
                // Se encontrou e corresponde, continua para o próximo critério
                continue;
            }

            // Se não encontrou no objeto raiz, verifica no objeto data
            if let Some(serde_json::Value::Object(data_obj)) = record_obj.get("data") {
                match data_obj.get(field) {
                    Some(actual_value) => {
                        // Verifica se o valor do campo corresponde ao critério com o operador especificado
                        if !compare_values(actual_value, expected_value, operator) {
                            println!("Campo '{}' no objeto data não corresponde com operador {:?}. Esperado: {:?}, Atual: {:?}", 
                                field, operator, expected_value, actual_value);
                            return false;
                        }
                    }
                    None => {
                        // Se o campo não existe no registro, retorna false
                        println!("Campo '{}' não encontrado no registro", field);
                        return false;
                    }
                }
            } else {
                // O registro não tem um campo "data" válido e o campo não foi encontrado no objeto raiz
                println!(
                    "Campo '{}' não encontrado no registro e não há objeto 'data' válido",
                    field
                );
                return false;
            }
        }
        // Todos os critérios foram atendidos
        println!("Registro corresponde a todos os critérios avançados");
        true
    } else {
        // O registro não é um objeto JSON
        println!("Registro não é um objeto JSON: {:?}", record);
        false
    }
}

/// Compara dois valores JSON usando o operador especificado
fn compare_values(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    operator: &ComparisonOperator,
) -> bool {
    match operator {
        ComparisonOperator::Eq => actual == expected,
        ComparisonOperator::Ne => actual != expected,
        ComparisonOperator::Gt => {
            match (actual, expected) {
                (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                    // Compara números
                    if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                        a_f64 > b_f64
                    } else {
                        false
                    }
                }
                (serde_json::Value::String(a), serde_json::Value::String(b)) => {
                    // Compara strings
                    a > b
                }
                _ => false,
            }
        }
        ComparisonOperator::Ge => match (actual, expected) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                    a_f64 >= b_f64
                } else {
                    false
                }
            }
            (serde_json::Value::String(a), serde_json::Value::String(b)) => a >= b,
            _ => false,
        },
        ComparisonOperator::Lt => match (actual, expected) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                    a_f64 < b_f64
                } else {
                    false
                }
            }
            (serde_json::Value::String(a), serde_json::Value::String(b)) => a < b,
            _ => false,
        },
        ComparisonOperator::Le => match (actual, expected) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                    a_f64 <= b_f64
                } else {
                    false
                }
            }
            (serde_json::Value::String(a), serde_json::Value::String(b)) => a <= b,
            _ => false,
        },
        ComparisonOperator::Contains => {
            match (actual, expected) {
                (serde_json::Value::String(a), serde_json::Value::String(b)) => {
                    // Verifica se a string contém a substring
                    a.contains(b)
                }
                (serde_json::Value::Array(a), _) => {
                    // Verifica se o array contém o valor
                    a.contains(expected)
                }
                _ => false,
            }
        }
        ComparisonOperator::StartsWith => {
            match (actual, expected) {
                (serde_json::Value::String(a), serde_json::Value::String(b)) => {
                    // Verifica se a string começa com a substring
                    a.starts_with(b)
                }
                _ => false,
            }
        }
        ComparisonOperator::EndsWith => {
            match (actual, expected) {
                (serde_json::Value::String(a), serde_json::Value::String(b)) => {
                    // Verifica se a string termina com a substring
                    a.ends_with(b)
                }
                _ => false,
            }
        }
    }
}
