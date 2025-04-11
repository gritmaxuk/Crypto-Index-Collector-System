mod none;
mod sma;
mod ema;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use crate::models::SmoothingType;

/// Trait for smoothing algorithms
pub trait SmoothingStrategy {
    /// Apply the smoothing algorithm to the price history
    fn apply(&self, price_history: &VecDeque<f64>, current_price: f64) -> f64;
}

/// Factory function to create smoothing algorithm instances
pub fn create_algorithm(smoothing_type: &SmoothingType) -> Box<dyn SmoothingStrategy> {
    match smoothing_type {
        SmoothingType::None => Box::new(none::NoSmoothing),
        SmoothingType::Sma => Box::new(sma::SimpleMovingAverage::new(20)),
        SmoothingType::Ema => Box::new(ema::ExponentialMovingAverage::new(20, 2.0)),
    }
}
