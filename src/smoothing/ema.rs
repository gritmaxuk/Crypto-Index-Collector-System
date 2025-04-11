use std::collections::VecDeque;
use super::SmoothingStrategy;

/// Exponential Moving Average smoothing algorithm
pub struct ExponentialMovingAverage {
    n: usize,  // Number of samples
    s: f64,    // Smoothing factor
}

impl ExponentialMovingAverage {
    pub fn new(n: usize, s: f64) -> Self {
        Self { n, s }
    }
}

impl SmoothingStrategy for ExponentialMovingAverage {
    fn apply(&self, price_history: &VecDeque<f64>, current_price: f64) -> f64 {
        if price_history.is_empty() {
            return current_price;
        }
        
        let a = self.s / (1.0 + self.n as f64);
        let previous_ema = price_history[0];
        
        current_price * a + previous_ema * (1.0 - a)
    }
}
