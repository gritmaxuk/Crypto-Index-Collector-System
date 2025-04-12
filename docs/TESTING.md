# Testing the Crypto Index Collector

This document provides instructions for testing the Crypto Index Collector application in various scenarios. These tests will help verify that the application is functioning correctly and meets the requirements.

**Note:** Docker deployment is currently not functional and is under development. Docker testing instructions will be added once Docker support is fully implemented.

## Prerequisites

Before running the tests, ensure you have the following:

- Rust and Cargo installed
- PostgreSQL with TimescaleDB extension (for database tests)

## Basic Functionality Testing

### 1. Running the Collector with Simple Configuration

This test verifies that the collector can start and fetch prices from a single exchange.

```bash
# Run the collector with the simple configuration
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.simple.toml
```

Expected output:
- The collector should start successfully
- You should see log messages showing price data being fetched from Coinbase
- No database operations should be performed (database is disabled in this config)

### 2. Running the Client to Connect to the Collector

This test verifies that the client can connect to the collector and receive index updates.

```bash
# In a separate terminal, run the client
RUST_LOG=info cargo run --bin crypto-index-client
```

Expected output:
- The client should connect to the collector
- You should see log messages showing index updates being received
- The index values should be displayed in the terminal

## Testing Different Configuration Files

### 1. Testing with Database Enabled

This test verifies that the collector can store price data in the database.

```bash
# Run the collector with the standard configuration (database enabled)
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml
```

Expected output:
- The collector should start successfully
- You should see log messages showing price data being fetched from multiple exchanges
- You should see log messages showing price data being stored in the database

### 2. Testing with Database Disabled

This test verifies that the collector can run without a database.

```bash
# Run the collector with the no-db configuration
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.no-db.toml
```

Expected output:
- The collector should start successfully
- You should see log messages showing price data being fetched from multiple exchanges
- No database operations should be performed

### 3. Testing with Disabled Feeds

This test verifies that the collector can run with some feeds disabled.

```bash
# Run the collector with the disabled-feeds configuration
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.disabled-feeds.toml
```

Expected output:
- The collector should start successfully
- You should see log messages showing price data being fetched only from enabled feeds
- The BTC-USD-INDEX should use only the Coinbase feed (100% weight)

## Testing Database Functionality

### 1. Verifying Data Storage

This test verifies that price data is being stored correctly in the database.

```bash
# Run the collector with database enabled
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml

# In a separate terminal, connect to the database and query the data
psql postgres://postgres:password@localhost:5432/crypto_indices -c "SELECT * FROM raw_price_data ORDER BY timestamp DESC LIMIT 10;"
```

Expected output:
- The query should return the 10 most recent price data points
- The data should include feed_id, price, and timestamp columns

### 2. Testing Retention Policy

This test verifies that the retention policy is working correctly.

```bash
# Connect to the database and check the retention policy
psql postgres://postgres:password@localhost:5432/crypto_indices -c "SELECT * FROM timescaledb_information.jobs WHERE hypertable_name = 'raw_price_data';"
```

Expected output:
- The query should return information about the retention policy job
- The retention period should match the configuration (30 days by default)

## Testing WebSocket Functionality

### 1. Testing WebSocket Connection

This test verifies that the WebSocket server is accepting connections.

```bash
# Run the collector
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml

# In a separate terminal, use wscat to connect to the WebSocket server
wscat -c ws://localhost:9000
```

Expected output:
- wscat should connect successfully
- You should receive a welcome message
- You should receive index updates periodically

### 2. Testing Multiple Clients

This test verifies that the WebSocket server can handle multiple clients.

```bash
# Run the collector
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml

# In separate terminals, run multiple clients
RUST_LOG=info cargo run --bin crypto-index-client
RUST_LOG=info cargo run --bin crypto-index-client
```

Expected output:
- Both clients should connect successfully
- Both clients should receive index updates
- The collector logs should show connections from multiple clients

## Testing Smoothing Algorithms

### 1. Testing No Smoothing

This test verifies that the "none" smoothing algorithm passes through raw values.

```bash
# Run the collector with a configuration that uses no smoothing
RUST_LOG=trace cargo run --bin crypto-index-collector -- --config config.simple.toml

# In a separate terminal, run the client
RUST_LOG=info cargo run --bin crypto-index-client
```

Expected output:
- The index values should closely match the raw price data
- There should be no smoothing effect visible

### 2. Testing Simple Moving Average (SMA)

This test verifies that the SMA smoothing algorithm works correctly.

```bash
# Create a temporary configuration file with SMA smoothing
cat > config.test-sma.toml << EOL
[feeds]
coinbase_btc_usd = { exchange = "coinbase", base_currency = "BTC", quote_currency = "USD", enabled = true }

[[indices]]
name = "BTC-USD-INDEX"
smoothing = "sma"
feeds = [
    { id = "coinbase_btc_usd", weight = 100 }
]

[database]
enabled = false

[websocket]
address = "0.0.0.0:9000"
EOL

# Run the collector with the SMA configuration
RUST_LOG=trace cargo run --bin crypto-index-collector -- --config config.test-sma.toml

# In a separate terminal, run the client
RUST_LOG=info cargo run --bin crypto-index-client
```

Expected output:
- The index values should be smoother than the raw price data
- After collecting 20 data points, the SMA should be fully operational
- The trace logs should show the SMA calculation details

### 3. Testing Exponential Moving Average (EMA)

This test verifies that the EMA smoothing algorithm works correctly.

```bash
# Create a temporary configuration file with EMA smoothing
cat > config.test-ema.toml << EOL
[feeds]
coinbase_btc_usd = { exchange = "coinbase", base_currency = "BTC", quote_currency = "USD", enabled = true }

[[indices]]
name = "BTC-USD-INDEX"
smoothing = "ema"
feeds = [
    { id = "coinbase_btc_usd", weight = 100 }
]

[database]
enabled = false

[websocket]
address = "0.0.0.0:9000"
EOL

# Run the collector with the EMA configuration
RUST_LOG=trace cargo run --bin crypto-index-collector -- --config config.test-ema.toml

# In a separate terminal, run the client
RUST_LOG=info cargo run --bin crypto-index-client
```

Expected output:
- The index values should be smoother than the raw price data but more responsive than SMA
- The trace logs should show the EMA calculation details

## Testing Graceful Shutdown

### 1. Testing Ctrl+C Shutdown

This test verifies that the collector shuts down gracefully when receiving a Ctrl+C signal.

```bash
# Run the collector
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml

# In a separate terminal, run the client
RUST_LOG=info cargo run --bin crypto-index-client

# Press Ctrl+C in the collector terminal
```

Expected output:
- The collector should log shutdown messages
- All tasks should terminate cleanly
- The client should detect the disconnection and attempt to reconnect

### 2. Testing Client Disconnection

This test verifies that the collector handles client disconnections gracefully.

```bash
# Run the collector
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml

# In a separate terminal, run the client
RUST_LOG=info cargo run --bin crypto-index-client

# Press Ctrl+C in the client terminal
```

Expected output:
- The client should send a close frame and terminate
- The collector should detect the client disconnection
- The collector should continue running without errors

## Docker Status

**Note:** Docker deployment is currently not functional and is under development. Docker testing will be added once Docker support is fully implemented.

## Troubleshooting

### Common Issues

1. **WebSocket Address Already in Use**

If you see an error like "Failed to bind WebSocket server: Address already in use", it means another process is using the WebSocket port. You can either:
- Stop the other process
- Change the WebSocket port in the configuration file

2. **Database Connection Failed**

If you see database connection errors, check that:
- PostgreSQL is running
- The TimescaleDB extension is installed
- The database credentials in the configuration file are correct

3. **Client Cannot Connect to Collector**

If the client cannot connect to the collector, check that:
- The collector is running
- The WebSocket server is bound to the correct address
- There are no firewall rules blocking the connection

## Automated Testing

The project includes unit tests for core functionality. Run them with:

```bash
cargo test
```

To run tests with verbose output:

```bash
cargo test -- --nocapture
```

To run a specific test:

```bash
cargo test test_name
```

## Performance Testing

For performance testing, you can use the following command to run the collector with a high number of indices and feeds:

```bash
# Create a performance test configuration
cat > config.performance.toml << EOL
# Define many feeds
[feeds]
coinbase_btc_usd = { exchange = "coinbase", symbol = "BTC-USD", enabled = true }
binance_btc_usdt = { exchange = "binance", symbol = "BTCUSDT", enabled = true }
coinbase_eth_usd = { exchange = "coinbase", symbol = "ETH-USD", enabled = true }
binance_eth_usdt = { exchange = "binance", symbol = "ETHUSDT", enabled = true }
coinbase_sol_usd = { exchange = "coinbase", symbol = "SOL-USD", enabled = true }
binance_sol_usdt = { exchange = "binance", symbol = "SOLUSDT", enabled = true }
coinbase_ada_usd = { exchange = "coinbase", symbol = "ADA-USD", enabled = true }
binance_ada_usdt = { exchange = "binance", symbol = "ADAUSDT", enabled = true }

# Define multiple indices
[[indices]]
name = "BTC-USD-INDEX"
smoothing = "ema"
feeds = [
    { id = "coinbase_btc_usd", weight = 60 },
    { id = "binance_btc_usdt", weight = 40 }
]

[[indices]]
name = "ETH-USD-INDEX"
smoothing = "sma"
feeds = [
    { id = "coinbase_eth_usd", weight = 50 },
    { id = "binance_eth_usdt", weight = 50 }
]

[[indices]]
name = "SOL-USD-INDEX"
smoothing = "none"
feeds = [
    { id = "coinbase_sol_usd", weight = 70 },
    { id = "binance_sol_usdt", weight = 30 }
]

[[indices]]
name = "ADA-USD-INDEX"
smoothing = "ema"
feeds = [
    { id = "coinbase_ada_usd", weight = 40 },
    { id = "binance_ada_usdt", weight = 60 }
]

[database]
enabled = false

[websocket]
address = "0.0.0.0:9000"
EOL

# Run the collector with the performance test configuration
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.performance.toml
```

Monitor CPU and memory usage to ensure the application performs well under load.
