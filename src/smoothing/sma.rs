use std::collections::VecDeque;
use super::SmoothingStrategy;

/// Simple Moving Average smoothing algorithm
pub struct SimpleMovingAverage {
    window_size: usize,
}

impl SimpleMovingAverage {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

impl SmoothingStrategy for SimpleMovingAverage {
    fn apply(&self, price_history: &VecDeque<f64>, current_price: f64) -> f64 {
        let mut sum = current_price;
        let mut count = 1;
        
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
