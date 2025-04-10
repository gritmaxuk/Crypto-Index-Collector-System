# Crypto Index Collector

A robust Rust application that collects real-time price data for specified cryptocurrencies (BTC, ETH) from multiple exchanges, calculates a configurable weighted average index with optional smoothing, and outputs this index data reliably.

## Project Overview

The Crypto Index Collector fetches cryptocurrency price data from multiple exchanges, applies configured weights to calculate a composite index, and optionally smooths the data using various algorithms. This creates a more reliable and stable price index compared to using a single exchange.

### Features

- Fetch real-time price data for BTC and ETH from Coinbase and Binance
- Calculate weighted average indices based on static configuration
- Support for multiple smoothing algorithms:
  - None (raw data)
  - Simple Moving Average (SMA)
  - Exponential Moving Average (EMA)
- Robust error handling with retry logic
- Structured logging for monitoring and notifications
- Docker containerization for easy deployment

## Getting Started

### Prerequisites

- Rust toolchain (1.70+)
- Docker (optional, for containerized deployment)

### Building from Source

1. Clone the repository
2. Build the application:

```bash
cargo build --release