mod models;

pub use models::{Config, DatabaseConfig, WebsocketConfig};

use crate::error::AppResult;
use std::path::Path;

/// Load configuration from a file
pub fn load_config<P: AsRef<Path>>(path: P) -> AppResult<Config> {
    Config::from_file(path).map_err(|e| e.to_string().into())
}
