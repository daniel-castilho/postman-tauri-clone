// src-tauri/src/domain/errors.rs
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Variable resolution failed: {0}")]
    VariableResolution(String),
    #[error("Script execution error: {0}")]
    ScriptError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, serde::Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl From<DomainError> for AppError {
    fn from(error: DomainError) -> Self {
        let code = match &error {
            DomainError::InvalidUrl(_) => "INVALID_URL",
            DomainError::VariableResolution(_) => "VARIABLE_RESOLUTION",
            DomainError::ScriptError(_) => "SCRIPT_ERROR",
            DomainError::AuthError(_) => "AUTH_ERROR",
            DomainError::NetworkError(_) => "NETWORK_ERROR",
            DomainError::ValidationError(_) => "VALIDATION_ERROR",
            DomainError::ConfigError(_) => "CONFIG_ERROR",
            DomainError::PersistenceError(_) => "PERSISTENCE_ERROR",
            DomainError::NotFound(_) => "NOT_FOUND",
            DomainError::SerializationError(_) => "SERIALIZATION_ERROR",
        }.to_string();

        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl AppError {
    pub fn persistence_error(message: impl Into<String>) -> Self {
        Self {
            code: "PERSISTENCE_ERROR".to_string(),
            message: message.into(),
        }
    }
}
