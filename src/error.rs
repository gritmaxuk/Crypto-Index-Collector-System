use std::error::Error;
use std::fmt;

/// Application-specific error type
#[derive(Debug)]
pub enum AppError {
    /// Configuration error
    Config(String),
    /// Database error
    Database(String),
    /// Exchange API error
    Exchange(String),
    /// WebSocket error
    WebSocket(String),
    /// Index calculation error
    IndexCalculation(String),
    /// I/O error
    Io(std::io::Error),
    /// Generic error
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Exchange(msg) => write!(f, "Exchange error: {}", msg),
            AppError::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            AppError::IndexCalculation(msg) => write!(f, "Index calculation error: {}", msg),
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Config(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for AppError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        AppError::WebSocket(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Exchange(err.to_string())
    }
}

impl From<std::num::ParseFloatError> for AppError {
    fn from(err: std::num::ParseFloatError) -> Self {
        AppError::Exchange(format!("Failed to parse price: {}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Other(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Other(err.to_string())
    }
}

/// Result type alias for AppError
pub type AppResult<T> = Result<T, AppError>;
