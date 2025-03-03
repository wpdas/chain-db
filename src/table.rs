use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::encryption::DataEncryption;
use crate::errors::ChainDBError;

const RECORDS_PER_FILE: usize = 1000;
pub const METADATA_FILE: &str = "metadata.cdb";

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
}
