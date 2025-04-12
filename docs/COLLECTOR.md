# Crypto Index Collector

The Crypto Index Collector is the core application that fetches price data from multiple exchanges, calculates weighted average indices with optional smoothing, and serves the results via WebSocket.

## Features

- Fetches real-time price data from Coinbase and Binance
- Supports BTC and ETH price feeds
- Calculates weighted average indices based on configuration
- Applies configurable smoothing algorithms:
  - None / passthru
  - 20-point Simple Moving Average (SMA)
  - 20-point Exponential Moving Average (EMA)
- Stores raw price data in PostgreSQL with TimescaleDB (optional)
- Serves calculated indices via WebSocket
- Robust error handling with retry logic
- Structured logging with clear distinction between data types

## Usage

```bash
# Run the collector
cargo run --bin crypto-index-collector

# Run with a specific configuration file
RUST_LOG=info cargo run --bin crypto-index-collector -- --config custom-config.toml
```

## Configuration

The collector is configured via a TOML file (`config.toml` by default). The configuration is organized into separate sections for feeds and indices. Here's a complete example with all available options:

```toml
# Define all feeds in one section
[feeds]
coinbase_btc_usd = { exchange = "coinbase", symbol = "BTC-USD", enabled = true }
binance_btc_usdt = { exchange = "binance", symbol = "BTCUSDT", enabled = true }
coinbase_eth_usd = { exchange = "coinbase", symbol = "ETH-USD", enabled = true }
binance_eth_usdt = { exchange = "binance", symbol = "ETHUSDT", enabled = true }

# Define indices and reference feeds
[[indices]]
name = "BTC-USD-INDEX"
smoothing = "ema"  # Options: "none", "sma", "ema"
feeds = [
    { id = "coinbase_btc_usd", weight = 60 },
    { id = "binance_btc_usdt", weight = 40 }
]

[[indices]]
name = "ETH-USD-INDEX"
smoothing = "sma"  # Options: "none", "sma", "ema"
feeds = [
    { id = "coinbase_eth_usd", weight = 50 },
    { id = "binance_eth_usdt", weight = 50 }
]

# Database configuration (optional)
[database]
enabled = true
url = "postgres://postgres:password@localhost:5432/crypto_indices"
retention_days = 30

# WebSocket server configuration
[websocket]
address = "0.0.0.0:9000"
```

### Configuration Options

#### Feeds Section

The `[feeds]` section defines all available price feeds:

```toml
[feeds]
feed_id = { exchange = "exchange_name", symbol = "symbol_name", enabled = true|false }
```

- `feed_id`: A unique identifier for the feed (e.g., `coinbase_btc_usd`)
- `exchange`: The exchange to fetch data from (`coinbase` or `binance`)
- `symbol`: The symbol to fetch (in exchange-specific format)
- `enabled`: Whether the feed is enabled (default: `true`)

#### Indices Section

The `[[indices]]` section defines the indices to calculate:

```toml
[[indices]]
name = "INDEX-NAME"
smoothing = "none|sma|ema"
feeds = [
    { id = "feed_id1", weight = 60 },
    { id = "feed_id2", weight = 40 }
]
```

- `name`: The name of the index (e.g., `BTC-USD-INDEX`)
- `smoothing`: The smoothing algorithm to use (`none`, `sma`, or `ema`)
- `feeds`: A list of feeds to include in the index
  - `id`: The ID of a feed defined in the `[feeds]` section
  - `weight`: The weight of the feed in the index (must sum to 100)

#### Database

- `enabled`: Whether to enable database persistence
- `url`: PostgreSQL connection URL
- `retention_days`: Number of days to retain data (uses TimescaleDB retention policy)

#### WebSocket

- `address`: Address and port for the WebSocket server (e.g., "127.0.0.1:9000")

## Logging

The collector uses structured logging with clear prefixes to distinguish between different types of data:

- `[RAW DATA]`: Raw price data fetched from exchanges
- `[CALCULATION]`: Index calculation details
- `[SMOOTHING]`: Smoothing algorithm application
- `[WEBSOCKET SEND]`: Data sent to WebSocket clients
- `[DATABASE]`: Database operations
- `[STARTUP]`, `[SHUTDOWN]`: System events

Example log output:

```
[2023-05-01T12:34:56Z INFO  crypto_index_collector] [STARTUP] Starting Crypto Index Collector...
[2023-05-01T12:34:56Z INFO  crypto_index_collector] [CONFIG] Configuration loaded successfully with 2 indices defined
[2023-05-01T12:34:57Z INFO  crypto_index_collector] [RAW DATA] Exchange: coinbase, Symbol: BTC-USD, Price: 42000.50, Time: 2023-05-01T12:34:57Z
[2023-05-01T12:34:57Z INFO  crypto_index_collector] [CALCULATION] Index: BTC-USD-INDEX, Raw Value: 42000.50
[2023-05-01T12:34:57Z INFO  crypto_index_collector] [SMOOTHING] Index: BTC-USD-INDEX, Algorithm: Ema, Raw: 42000.50, Smoothed: 42000.50, Diff: 0.0000%
[2023-05-01T12:34:57Z INFO  crypto_index_collector] [WEBSOCKET SEND] Client: 127.0.0.1:54321, Index: BTC-USD-INDEX, Value: 42000.50
```

## Error Handling

The collector implements robust error handling:

- Retries failed API calls with exponential backoff
- Logs warnings after 5 consecutive failures to fetch price data
- Gracefully handles WebSocket connection failures
- Continues operation even if some price feeds are unavailable

## Database Schema

When database persistence is enabled, the collector creates the following schema:

```sql
CREATE TABLE raw_price_data (
    id SERIAL,
    feed_id TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (id, timestamp)
);

-- Convert to TimescaleDB hypertable
SELECT create_hypertable('raw_price_data', 'timestamp');

-- Create indexes
CREATE INDEX idx_raw_price_data_timestamp ON raw_price_data (timestamp);
CREATE UNIQUE INDEX idx_raw_price_data_feed_timestamp ON raw_price_data (feed_id, timestamp);
```

## Testing

The collector can be tested in various configurations to verify different aspects of its functionality:

```bash
# Test with simple configuration (single feed, no database)
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.simple.toml

# Test with database enabled
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.toml

# Test with database disabled
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.no-db.toml

# Test with specific smoothing algorithm (SMA)
RUST_LOG=trace cargo run --bin crypto-index-collector -- --config config.test-sma.toml

# Test with specific smoothing algorithm (EMA)
RUST_LOG=trace cargo run --bin crypto-index-collector -- --config config.test-ema.toml

# Test with performance configuration (multiple indices and feeds)
RUST_LOG=info cargo run --bin crypto-index-collector -- --config config.performance.toml
```

For detailed testing instructions covering various scenarios, see the [Testing Guide](TESTING.md).

## Limitations

- The collector uses static configuration and doesn't support in-flight changes to indices or feeds
- Only Coinbase and Binance exchanges are supported (Kraken is not implemented)
- Smoothing parameters (window size, smoothing factor) are fixed and not configurable
