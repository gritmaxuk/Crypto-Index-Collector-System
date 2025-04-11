use std::collections::VecDeque;
use super::SmoothingStrategy;

/// Exponential Moving Average smoothing algorithm
pub struct ExponentialMovingAverage {
    n: usize,  // Number of samples
    s: f64,    // Smoothing factor
}

impl ExponentialMovingAverage {
    pub fn new(n: usize, s: f64) -> Self {
        // Ensure n is at least 1 to avoid division by zero
        let n = if n == 0 { 1 } else { n };
        // Ensure s is non-negative
        let s = s.max(0.0);
        Self { n, s }
    }
}

impl SmoothingStrategy for ExponentialMovingAverage {
    fn apply(&self, price_history: &VecDeque<f64>, current_price: f64) -> f64 {
        // If history is empty, return current price
        if price_history.is_empty() {
            return current_price;
        }

        // Calculate alpha (smoothing factor)
        let a = self.s / (1.0 + self.n as f64);

        // Handle edge cases
        if a >= 1.0 {
            // If alpha is 1 or greater, just return current price
            return current_price;
        } else if a <= 0.0 {
            // If alpha is 0 or negative, just return previous EMA
            return price_history[0];
        }

        // Get the previous EMA (most recent price in history)
        let previous_ema = price_history[0];

        // Calculate EMA: current_price * alpha + previous_ema * (1 - alpha)
        current_price * a + previous_ema * (1.0 - a)
    }
}
