use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::encryption::DataEncryption;
use crate::errors::ChainDBError;

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
        let record_json = serde_json::to_vec(&record)?;
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

        Ok(())
    }

    pub fn update(&mut self, record: &T) -> Result<(), ChainDBError> {
        if self.metadata.total_records == 0 {
            return self.persist(record);
        }

        let record_json = serde_json::to_vec(&record)?;
        let encrypted_record = self.encryption.encrypt(&record_json)?;

        let file_name = format!("data_{}.cdb", self.metadata.current_file);
        let file_path = self.path.join(&file_name);

        // Read all lines except the last one
        let content = fs::read_to_string(&file_path)?;
        let mut lines: Vec<_> = content.lines().collect();

        // Replace or append the last record
        if !lines.is_empty() {
            lines.pop(); // Remove last line
        }
        let base64_str = general_purpose::STANDARD.encode(&encrypted_record);
        lines.push(&base64_str);

        // Write back to file
        let mut file = fs::File::create(&file_path)?;
        for line in lines {
            writeln!(file, "{}", line)?;
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
    /// # Exemplo
    ///
    /// ```
    /// let mut criteria = HashMap::new();
    /// criteria.insert("nome".to_string(), serde_json::json!("João"));
    /// criteria.insert("idade".to_string(), serde_json::json!(30));
    /// let resultados = tabela.findWhere(criteria, Some(10), true)?;
    /// ```
    pub fn findWhere(
        &self,
        criteria: HashMap<String, serde_json::Value>,
        limit: Option<usize>,
        reverse: bool,
    ) -> Result<Vec<T>, ChainDBError> {
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

    /// Versão simplificada da função findWhere que usa valores padrão.
    ///
    /// Esta função é uma conveniência que chama findWhere com reverse=true.
    ///
    /// # Argumentos
    ///
    /// * `criteria` - Um HashMap contendo os critérios de busca
    /// * `limit` - Número máximo de registros a serem retornados (opcional)
    ///
    /// # Retornos
    ///
    /// * `Ok(Vec<T>)` - Um vetor contendo os registros que correspondem aos critérios
    /// * `Err(ChainDBError)` - Se ocorrer um erro durante a busca
    pub fn findWhere_default(
        &self,
        criteria: HashMap<String, serde_json::Value>,
        limit: Option<usize>,
    ) -> Result<Vec<T>, ChainDBError> {
        self.findWhere(criteria, limit, true)
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
    /// # Exemplo
    ///
    /// ```
    /// let mut criteria = HashMap::new();
    /// criteria.insert("idade".to_string(), (ComparisonOperator::Gt, serde_json::json!(30)));
    /// criteria.insert("nome".to_string(), (ComparisonOperator::Contains, serde_json::json!("Silva")));
    /// let resultados = tabela.findWhereAdvanced(criteria, Some(10), true)?;
    /// ```
    pub fn findWhereAdvanced(
        &self,
        criteria: HashMap<String, (ComparisonOperator, serde_json::Value)>,
        limit: Option<usize>,
        reverse: bool,
    ) -> Result<Vec<T>, ChainDBError> {
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

    /// Versão simplificada da função findWhereAdvanced que usa valores padrão.
    ///
    /// Esta função é uma conveniência que chama findWhereAdvanced com reverse=true.
    ///
    /// # Argumentos
    ///
    /// * `criteria` - Um HashMap contendo os critérios de busca avançados
    /// * `limit` - Número máximo de registros a serem retornados (opcional)
    ///
    /// # Retornos
    ///
    /// * `Ok(Vec<T>)` - Um vetor contendo os registros que correspondem aos critérios
    /// * `Err(ChainDBError)` - Se ocorrer um erro durante a busca
    pub fn findWhereAdvanced_default(
        &self,
        criteria: HashMap<String, (ComparisonOperator, serde_json::Value)>,
        limit: Option<usize>,
    ) -> Result<Vec<T>, ChainDBError> {
        self.findWhereAdvanced(criteria, limit, true)
    }
}

/// Função auxiliar para verificar se um registro corresponde aos critérios especificados
fn matches_criteria(
    record: &serde_json::Value,
    criteria: &HashMap<String, serde_json::Value>,
) -> bool {
    if let serde_json::Value::Object(record_obj) = record {
        // Verifica se existe o campo "data" e se é um objeto
        if let Some(serde_json::Value::Object(data_obj)) = record_obj.get("data") {
            // Verifica cada critério
            for (field, expected_value) in criteria {
                match data_obj.get(field) {
                    Some(actual_value) => {
                        // Se o valor do campo não corresponde ao critério, retorna false
                        if actual_value != expected_value {
                            println!(
                                "Campo '{}' não corresponde. Esperado: {:?}, Atual: {:?}",
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
            }
            // Todos os critérios foram atendidos
            println!("Registro corresponde a todos os critérios");
            true
        } else {
            // O registro não tem um campo "data" válido
            println!(
                "Registro não possui um campo 'data' válido: {:?}",
                record_obj
            );
            false
        }
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
        // Verifica se existe o campo "data" e se é um objeto
        if let Some(serde_json::Value::Object(data_obj)) = record_obj.get("data") {
            // Verifica cada critério
            for (field, (operator, expected_value)) in criteria {
                match data_obj.get(field) {
                    Some(actual_value) => {
                        // Verifica se o valor do campo corresponde ao critério com o operador especificado
                        if !compare_values(actual_value, expected_value, operator) {
                            println!("Campo '{}' não corresponde com operador {:?}. Esperado: {:?}, Atual: {:?}", 
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
            }
            // Todos os critérios foram atendidos
            println!("Registro corresponde a todos os critérios avançados");
            true
        } else {
            // O registro não tem um campo "data" válido
            println!(
                "Registro não possui um campo 'data' válido: {:?}",
                record_obj
            );
            false
        }
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
