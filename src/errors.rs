use base64;
use serde_json;
use std::io;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum ChainDBError {
    IoError(io::Error),
    SerializationError(String),
    InvalidCredentials(String),
    EncryptionError(String),
    DecryptionError(String),
    DatabaseAlreadyExists(String),
    DatabaseNotFound(String),
    ConfigNotFound(String),
    ValidationError(String),
    RecordNotFound(String),
}

impl From<io::Error> for ChainDBError {
    fn from(error: io::Error) -> Self {
        ChainDBError::IoError(error)
    }
}

impl From<serde_json::Error> for ChainDBError {
    fn from(error: serde_json::Error) -> Self {
        ChainDBError::SerializationError(error.to_string())
    }
}

impl From<String> for ChainDBError {
    fn from(error: String) -> Self {
        ChainDBError::EncryptionError(error)
    }
}

impl From<&str> for ChainDBError {
    fn from(error: &str) -> Self {
        ChainDBError::EncryptionError(error.to_string())
    }
}

impl From<FromUtf8Error> for ChainDBError {
    fn from(error: FromUtf8Error) -> Self {
        ChainDBError::SerializationError(error.to_string())
    }
}

impl From<base64::DecodeError> for ChainDBError {
    fn from(error: base64::DecodeError) -> Self {
        ChainDBError::DecryptionError(error.to_string())
    }
}

impl std::fmt::Display for ChainDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainDBError::IoError(e) => write!(f, "IO error: {}", e),
            ChainDBError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ChainDBError::InvalidCredentials(e) => write!(f, "Invalid credentials: {}", e),
            ChainDBError::EncryptionError(e) => write!(f, "Encryption error: {}", e),
            ChainDBError::DecryptionError(e) => write!(f, "Decryption error: {}", e),
            ChainDBError::DatabaseAlreadyExists(name) => {
                write!(f, "Database already exists: {}", name)
            }
            ChainDBError::DatabaseNotFound(name) => write!(f, "Database not found: {}", name),
            ChainDBError::ConfigNotFound(name) => {
                write!(f, "Config not found for database: {}", name)
            }
            ChainDBError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ChainDBError::RecordNotFound(name) => write!(f, "Record not found: {}", name),
        }
    }
}

impl std::error::Error for ChainDBError {}
