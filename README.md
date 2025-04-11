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
- Automatic restart on failure with notification system
- WebSocket server for real-time index updates
- Command-line client for consuming index data

## Getting Started

### Prerequisites

- Rust toolchain (1.70+)
- Docker (optional, for containerized deployment)

### Building from Source

1. Clone the repository
2. Build the application:

```bash
cargo build --release
```

## Components

The project consists of three main components:

### 1. Index Collector (Main Application)

The core application that fetches price data, calculates indices, and serves them via WebSocket.

```bash
# Run the main application
cargo run --bin crypto-index-collector
```

See the configuration options in `config.toml`.

### 2. WebSocket Client

A command-line client that connects to the WebSocket server and displays index updates.

```bash
# Run the client
cargo run --bin crypto-index-client
```

For more details, see [client-README.md](client-README.md).

### 3. Supervisor

A process that monitors the main application, automatically restarts it on failure, and sends notifications.

```bash
# Run the supervisor
cargo run --bin crypto-index-supervisor
```

For more details, see [supervisor-README.md](supervisor-README.md).

## Reliability Features

### Automatic Restart

The supervisor monitors the main application and automatically restarts it if it crashes or exits with an error. Key features:

- Exponential backoff for restart attempts
- Configurable restart limits and monitoring periods
- Notification system for restart events

### Error Handling

- Retry logic for exchange API calls
- Notification after 5 consecutive failures to fetch price data
- Graceful handling of WebSocket connection failures

### Logging

Structured logging with clear prefixes to distinguish between:

- Raw data from exchanges
- Calculated index values
- Data sent to WebSocket clients
- Database operations
- System events (startup, shutdown, etc.)