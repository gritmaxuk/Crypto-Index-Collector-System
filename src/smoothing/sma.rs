use std::collections::VecDeque;
use super::SmoothingStrategy;

/// Simple Moving Average smoothing algorithm
pub struct SimpleMovingAverage {
    window_size: usize,
}

impl SimpleMovingAverage {
    pub fn new(window_size: usize) -> Self {
        // Ensure window size is at least 1
        let window_size = if window_size == 0 { 1 } else { window_size };
        Self { window_size }
    }
}

impl SmoothingStrategy for SimpleMovingAverage {
    fn apply(&self, price_history: &VecDeque<f64>, current_price: f64) -> f64 {
        // If window size is 1 or history is empty, just return current price
        if self.window_size == 1 || price_history.is_empty() {
            return current_price;
        }

        let mut sum = current_price;
        let mut count = 1;

        // Add prices from history up to window_size - 1
        for (i, &price) in price_history.iter().enumerate() {
            if i < self.window_size - 1 {
                sum += price;
                count += 1;
            } else {
                break;
            }
        }

        sum / count as f64
    }
}
