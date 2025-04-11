// Re-export modules for external use
pub mod config;
pub mod exchange;
pub mod index;
pub mod storage;
pub mod smoothing;
pub mod websocket;
pub mod notification;
pub mod logging;
pub mod models;
pub mod error;

// Export commonly used types for convenience
pub use models::{FeedData, PriceFeed, IndexDefinition, SmoothingType};
pub use index::calculator::IndexCalculator;
pub use index::models::IndexResult;
pub use exchange::traits::Exchange;
pub use error::AppError;
