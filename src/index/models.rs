use chrono::{DateTime, Utc};

/// Result of an index calculation
#[derive(Debug, Clone)]
pub struct IndexResult {
    /// Name of the index
    pub name: String,
    /// Timestamp of the calculation
    pub timestamp: DateTime<Utc>,
    /// Calculated index value
    pub value: f64,
}
