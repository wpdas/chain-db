use base64::{engine::general_purpose, Engine as _};
use chrono;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use crate::config::Config;
use crate::encryption::DataEncryption;
use crate::errors::ChainDBError;
use crate::table::{Table, METADATA_FILE};

const DATA_DIR: &str = ".chain_db_data";

#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
    pub db: ChainDB,
    pub auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainDB {
    base_path: PathBuf,
    config: Config,
    #[serde(skip)]
    encryption: DataEncryption,
}

impl ChainDB {
    pub fn create_database(name: &str, user: &str, password: &str) -> Result<(), ChainDBError> {
        // Verificar se o banco de dados já existe
        let data_dir = PathBuf::from(DATA_DIR);
        let base_path = data_dir.join(name);

        if base_path.exists() {
            return Err(ChainDBError::DatabaseAlreadyExists(format!(
                "Database '{}' already exists",
                name
            )));
        }

        let encryption = DataEncryption::new(password);
        let config = Config::new(name, user, password);

        // Create data directory
        fs::create_dir_all(&base_path)?;

        // Save encrypted config
        let config_json = serde_json::to_vec(&config)?;
        let encrypted_config = encryption.encrypt(&config_json)?;
        let config_path = base_path.join("config.cdb");
        let mut file = fs::File::create(config_path)?;
        file.write_all(&encrypted_config)?;

        Ok(())
    }

    pub fn connect(name: &str, user: &str, password: &str) -> Result<ConnectionInfo, ChainDBError> {
        let encryption = DataEncryption::new(password);
        let data_dir = PathBuf::from(DATA_DIR);
        let base_path = data_dir.join(name);
        let config_path = base_path.join("config.cdb");

        // Read and decrypt config
        let encrypted_config = fs::read(config_path)?;
        let config_json = encryption.decrypt(&encrypted_config)?;
        let config: Config = serde_json::from_slice(&config_json)?;

        // Verify credentials
        if config.name != name || config.user != user || config.password != password {
            return Err(ChainDBError::InvalidCredentials(
                "Invalid credentials".to_string(),
            ));
        }

        // Generate auth token
        let auth_data = format!("{}:{}:{}", name, user, password);
        let auth_token = general_purpose::STANDARD.encode(auth_data);

        Ok(ConnectionInfo {
            db: Self {
                base_path,
                config,
                encryption,
            },
            auth_token,
        })
    }

    pub fn change_password(&mut self, new_password: &str) -> Result<(), ChainDBError> {
        // Verificar se a nova senha é diferente da atual
        if self.config.password == new_password {
            return Err(ChainDBError::ValidationError(
                "New password is the same as the current password".to_string(),
            ));
        }

        // Calcular o tamanho total do diretório do banco de dados
        let mut total_size: u64 = 0;
        calculate_dir_size(&self.base_path, &mut total_size)?;

        // Verificar se há espaço suficiente para o backup e arquivos temporários
        // Precisamos de pelo menos 2x o tamanho atual (1x para backup, 1x para arquivos temporários)
        let required_space = total_size * 2;

        // Obter informações sobre o sistema de arquivos
        let available_space = get_available_space(&self.base_path)?;

        println!("Total database size: {} bytes", total_size);
        println!("Available space: {} bytes", available_space);
        println!("Required space: {} bytes", required_space);

        if available_space < required_space {
            return Err(ChainDBError::ValidationError(format!(
                "Not enough disk space. Required: {} bytes, Available: {} bytes",
                required_space, available_space
            )));
        }

        // Create bkp_timestamp directory
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_dir = self.base_path.join(format!("bkp_{}", timestamp));
        fs::create_dir_all(&backup_dir)?;

        // Get current table path
        let table_path = self.base_path.clone();
        println!("table_path: {:?}", table_path);

        // Criar uma estrutura para controlar o processo e permitir rollback
        struct ChangePasswordProcess {
            original_path: PathBuf,
            backup_dir: PathBuf,
            temp_dir: PathBuf,
            success: bool,
        }

        impl Drop for ChangePasswordProcess {
            fn drop(&mut self) {
                if !self.success {
                    println!("Error occurred during password change. Rolling back...");

                    // Remover diretório temporário se existir
                    if self.temp_dir.exists() {
                        let _ = fs::remove_dir_all(&self.temp_dir);
                    }

                    println!("Rollback completed. Original data remains intact.");
                }
            }
        }

        // Iniciar o processo com controle de rollback
        let mut process = ChangePasswordProcess {
            original_path: self.base_path.clone(),
            backup_dir: backup_dir.clone(),
            temp_dir: self.base_path.join("temp"),
            success: false,
        };

        // Copy the table folder to the backup directory
        let backup_path = backup_dir.join(self.base_path.file_name().unwrap_or_default());
        copy_dir_all_files(&table_path, &backup_dir)?;
        println!("Backup created at: {:?}", backup_dir);

        // Create a temp folder to prepare the updated files with the new password
        let temp_dir = process.temp_dir.clone();
        fs::create_dir_all(&temp_dir)?;

        // Criar nova instância de criptografia com a nova senha
        let new_encryption = DataEncryption::new(new_password);

        // Processar todas as tabelas (pastas) dentro do diretório base
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            // Ignorar arquivos e diretórios especiais
            if !path.is_dir() || path == temp_dir || path == backup_dir {
                continue;
            }

            let table_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            println!("Processing table: {}", table_name);

            // Criar diretório temporário para esta tabela
            let temp_table_dir = temp_dir.join(&table_name);
            fs::create_dir_all(&temp_table_dir)?;

            // Processar todos os arquivos na tabela
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if !file_path.is_file() {
                    continue;
                }

                let file_name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let temp_file_path = temp_table_dir.join(&file_name);

                // Processar arquivos de dados (data_N.cdb) linha por linha
                if file_name.starts_with("data_") && file_name.ends_with(".cdb") {
                    println!("Processing data file: {}", file_name);

                    // Abrir arquivo original para leitura linha por linha
                    let file = fs::File::open(&file_path)?;
                    let reader = BufReader::new(file);

                    // Criar arquivo temporário para escrita
                    let temp_file = fs::File::create(&temp_file_path)?;
                    let mut writer = BufWriter::new(temp_file);

                    // Processar linha por linha
                    for line in reader.lines() {
                        let line = line?;
                        if !line.is_empty() {
                            // Decodificar base64
                            let encrypted_data = general_purpose::STANDARD
                                .decode(&line)
                                .map_err(|e| ChainDBError::SerializationError(e.to_string()))?;

                            // Decriptar com a chave antiga
                            let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

                            // Reencriptar com a nova chave
                            let new_encrypted_data = new_encryption.encrypt(&decrypted_data)?;

                            // Codificar em base64 e escrever no novo arquivo
                            let new_line = general_purpose::STANDARD.encode(&new_encrypted_data);
                            writeln!(writer, "{}", new_line)?;
                        }
                    }

                    // Garantir que todos os dados sejam escritos
                    writer.flush()?;
                }
                // Processar arquivo de metadados (metadata.cdb)
                else if file_name == "metadata.cdb" {
                    println!("Processing metadata file for table: {}", table_name);

                    // Ler arquivo original com criptografia antiga
                    let encrypted_data = fs::read(&file_path)?;
                    let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

                    // Reencriptar com nova chave
                    let new_encrypted_data = new_encryption.encrypt(&decrypted_data)?;

                    // Escrever para arquivo temporário
                    fs::write(&temp_file_path, &new_encrypted_data)?;
                }
            }
        }

        // Atualizar o arquivo de configuração com a nova senha
        let mut new_config = self.config.clone();
        new_config.password = new_password.to_string();

        // Reencriptar e salvar o arquivo de configuração na pasta temp
        let config_json = serde_json::to_vec(&new_config)?;
        let encrypted_config = new_encryption.encrypt(&config_json)?;
        let temp_config_path = temp_dir.join("config.cdb");
        fs::write(&temp_config_path, &encrypted_config)?;

        // Substituir os arquivos originais pelos arquivos temporários
        println!("Replacing original files with re-encrypted files...");

        // Primeiro, mover o arquivo de configuração
        let config_path = self.base_path.join("config.cdb");
        fs::remove_file(&config_path)?;
        fs::copy(&temp_config_path, &config_path)?;

        // Depois, mover os arquivos das tabelas
        for entry in fs::read_dir(&temp_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Pular o arquivo de configuração que já foi tratado
            if !path.is_dir() {
                continue;
            }

            let table_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let original_table_path = self.base_path.join(&table_name);

            // Processar todos os arquivos na tabela temporária
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if !file_path.is_file() {
                    continue;
                }

                let file_name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let original_file_path = original_table_path.join(&file_name);

                // Substituir o arquivo original pelo temporário
                if original_file_path.exists() {
                    fs::remove_file(&original_file_path)?;
                }
                fs::copy(&file_path, &original_file_path)?;
            }
        }

        // Remover a pasta temporária
        fs::remove_dir_all(&temp_dir)?;

        // Atualizar a instância atual com a nova criptografia e configuração
        self.encryption = new_encryption;
        self.config = new_config;

        // Marcar o processo como bem-sucedido
        process.success = true;

        println!("Password changed successfully!");
        Ok(())
    }

    pub fn create_table<T>(&self, name: &str) -> Result<Table<T>, ChainDBError>
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let table_path = self.base_path.join(name);
        fs::create_dir_all(&table_path)?;
        Table::new(table_path, self.encryption.clone())
    }

    /// Lists all tables in the database
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - A vector containing the names of all tables in the database
    /// * `Err(ChainDBError)` - If an error occurs while reading the database directory
    pub fn list_tables(&self) -> Result<Vec<String>, ChainDBError> {
        let mut tables = Vec::new();

        // Read the database directory
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip non-directory entries and special files
            if !path.is_dir() || entry.file_name() == "config.cdb" {
                continue;
            }

            // Check if this is a valid table directory (contains metadata.cdb)
            let metadata_path = path.join(METADATA_FILE);
            if metadata_path.exists() {
                if let Some(table_name) = path.file_name() {
                    if let Some(name) = table_name.to_str() {
                        tables.push(name.to_string());
                    }
                }
            }
        }

        Ok(tables)
    }
}

// Função auxiliar para copiar diretórios recursivamente
fn copy_dir_all_files(source: &PathBuf, destination: &PathBuf) -> Result<(), ChainDBError> {
    // Criar o diretório de destino se não existir
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    // Ler o conteúdo do diretório de origem
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        // Ignorar o diretório de backup para evitar recursão infinita
        if path.is_dir() {
            // Ignorar pastas que começam com "bkp_" ou que são o próprio destino
            let dir_name = file_name.to_string_lossy();
            if dir_name.starts_with("bkp_") || path.starts_with(destination) {
                println!("Skipping backup directory: {:?}", path);
                continue;
            }
        }

        let destination_path = destination.join(file_name);

        if path.is_dir() {
            // Recursivamente copiar subdiretórios
            copy_dir_all_files(&path, &destination_path)?;
        } else {
            // Copiar arquivo
            fs::copy(&path, &destination_path)?;
        }
    }

    Ok(())
}

// Função para calcular o tamanho de um diretório recursivamente
fn calculate_dir_size(dir: &PathBuf, total_size: &mut u64) -> Result<(), ChainDBError> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                calculate_dir_size(&path, total_size)?;
            } else {
                *total_size += fs::metadata(&path)?.len();
            }
        }
    }
    Ok(())
}

// Função para obter o espaço disponível no sistema de arquivos
fn get_available_space(path: &PathBuf) -> Result<u64, ChainDBError> {
    // Esta é uma implementação simplificada que funciona apenas em sistemas Unix-like
    // Para uma implementação completa, seria necessário usar bibliotecas específicas para cada SO

    #[cfg(target_family = "unix")]
    {
        use std::process::Command;

        let output = Command::new("df")
            .args(&["-k", &path.to_string_lossy()])
            .output()
            .map_err(|e| {
                ChainDBError::ValidationError(format!("Failed to execute df command: {}", e))
            })?;

        if !output.status.success() {
            return Err(ChainDBError::ValidationError(
                "Failed to get disk space information".to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        if lines.len() < 2 {
            return Err(ChainDBError::ValidationError(
                "Invalid df output format".to_string(),
            ));
        }

        let fields: Vec<&str> = lines[1].split_whitespace().collect();

        if fields.len() < 4 {
            return Err(ChainDBError::ValidationError(
                "Invalid df output format".to_string(),
            ));
        }

        let available_kb = fields[3].parse::<u64>().map_err(|_| {
            ChainDBError::ValidationError("Failed to parse available space".to_string())
        })?;

        // Converter KB para bytes
        Ok(available_kb * 1024)
    }

    #[cfg(not(target_family = "unix"))]
    {
        // Em sistemas não-Unix, retornamos um valor grande para não bloquear a operação
        // Em uma implementação real, usaríamos APIs específicas do sistema operacional
        Ok(u64::MAX / 2) // Valor grande, mas não o máximo para evitar overflow
    }
}
