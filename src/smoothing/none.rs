use std::collections::VecDeque;
use super::SmoothingStrategy;

/// No smoothing - returns the raw price
pub struct NoSmoothing;

impl SmoothingStrategy for NoSmoothing {
    fn apply(&self, _price_history: &VecDeque<f64>, current_price: f64) -> f64 {
        current_price
    }
}
