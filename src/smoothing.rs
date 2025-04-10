use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum SmoothingAlgorithm {
    None,
    SimpleMovingAverage(usize),
    ExponentialMovingAverage { n: usize, s: f64 },
}

impl SmoothingAlgorithm {
    pub fn apply(&self, price_history: &VecDeque<f64>, current_price: f64) -> f64 {
        match self {
            SmoothingAlgorithm::None => current_price,
            SmoothingAlgorithm::SimpleMovingAverage(window_size) => {
                let mut sum = current_price;
                let mut count = 1;
                
                for (i, &price) in price_history.iter().enumerate() {
                    if i < *window_size - 1 {
                        sum += price;
                        count += 1;
                    } else {
                        break;
                    }
                }
                
                sum / count as f64
            },
            SmoothingAlgorithm::ExponentialMovingAverage { n, s } => {
                if price_history.is_empty() {
                    return current_price;
                }
                
                let a = *s / (1.0 + *n as f64);
                let previous_ema = price_history[0];
                
                current_price * a + previous_ema * (1.0 - a)
            }
        }
    }
}

impl From<&crate::models::SmoothingType> for SmoothingAlgorithm {
    fn from(smoothing_type: &crate::models::SmoothingType) -> Self {
        match smoothing_type {
            crate::models::SmoothingType::None => SmoothingAlgorithm::None,
            crate::models::SmoothingType::Sma => SmoothingAlgorithm::SimpleMovingAverage(20),
            crate::models::SmoothingType::Ema => SmoothingAlgorithm::ExponentialMovingAverage { n: 20, s: 2.0 },
        }
    }
}