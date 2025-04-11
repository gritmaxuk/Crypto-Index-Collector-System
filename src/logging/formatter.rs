use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use crate::error::AppResult;

/// Set up structured logging for the application
pub fn setup_logging() -> AppResult<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("Failed to set up logging: {}", e).into())
}
