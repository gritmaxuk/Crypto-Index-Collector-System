# Crypto Index Collector

A robust Rust application that collects real-time price data for specified cryptocurrencies (BTC, ETH) from multiple exchanges, calculates a configurable weighted average index with optional smoothing, and outputs this index data reliably via WebSocket.

## Project Overview

The Crypto Index Collector fetches cryptocurrency price data from multiple exchanges, applies configured weights to calculate a composite index, and optionally smooths the data using various algorithms. This creates a more reliable and stable price index compared to using a single exchange.

### Features

- **Exchange Support**: Fetch real-time price data from Coinbase and Binance (Kraken not implemented)
- **Asset Support**: BTC and ETH price feeds
- **Index Calculation**: Weighted average based on configurable weights
- **Smoothing Algorithms**:
  - None / passthru
  - 20-point Simple Moving Average (SMA)
  - 20-point Exponential Moving Average (EMA)
- **Data Persistence**: Optional PostgreSQL database with TimescaleDB for storing price data
- **WebSocket Server**: Real-time streaming of calculated indices
- **Reliability Features**:
  - Automatic restart on failure with notification system
  - Exponential backoff for reconnection attempts
  - Notification after 5 consecutive failures to fetch price data
- **Structured Logging**: Clear distinction between raw data, calculated data, and data sent to WebSocket
- **Client Application**: Command-line client for consuming index data

### Requirements Coverage

| Requirement | Status | Notes |
|-------------|--------|-------|
| Container/Virtual Environment | ✅ | Docker support implemented |
| Configuration | ⚠️ | Using TOML instead of etcd for simplicity |
| Auto-restart on failure | ✅ | Implemented with supervisor |
| Notification on restart | ✅ | Console and script-based notifications |
| Notification on feed failures | ✅ | After 5 consecutive failures |
| Notification on index failures | ✅ | Implemented via logging |
| Dynamic index management | ❌ | Not implemented (add/remove/pause) |
| Dynamic feed management | ❌ | Not implemented (add/remove) |
| Exchange support | ⚠️ | Coinbase, Binance implemented; Kraken missing |
| Asset support | ✅ | BTC, ETH implemented |
| Configurable weights | ✅ | Implemented via configuration |
| Smoothing algorithms | ✅ | All required algorithms implemented |

## Getting Started

### Prerequisites

- Rust toolchain (1.70+)
- Docker and Docker Compose (optional, for containerized deployment)
- PostgreSQL with TimescaleDB extension (optional, for data persistence)

### Building from Source

1. Clone the repository
2. Build the application:

```bash
cargo build --release
```

### Running with Docker

The project includes Docker support for easy deployment:

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

See [DOCKER.md](docs/DOCKER.md) for detailed documentation on Docker deployment.

### Configuration

The application is configured via the `config.toml` file. Key configuration options:

```toml
# Example configuration
[[indices]]
name = "BTC-USD-INDEX"
symbol = "BTC/USD"
smooothing = "ema"  # Options: "none", "sma", "ema"

[[indices.feeds]]
id = "coinbase-btc-usd"
exchange = "coinbase"
symbol = "BTC-USD"
weight = 60  # Percentage weight (must sum to 100)

[[indices.feeds]]
id = "binance-btc-usdt"
exchange = "binance"
symbol = "BTCUSDT"
weight = 40

[database]
enabled = true
url = "postgres://postgres:password@localhost:5432/crypto_indices"
retention_days = 30

[websocket]
address = "127.0.0.1:9000"
```

## Components

The project consists of three main components in a client-server architecture:

### 1. Index Collector (Main Application)

The core application that fetches price data, calculates indices, and serves them via WebSocket.

```bash
# Run the main application
cargo run --bin crypto-index-collector
```

The collector performs the following functions:
- Fetches price data from configured exchanges (Coinbase, Binance)
- Applies smoothing algorithms as specified in the configuration
- Calculates weighted average indices
- Optionally stores raw price data in a PostgreSQL database
- Serves calculated indices via WebSocket

See [COLLECTOR.md](docs/COLLECTOR.md) for detailed documentation.

### 2. WebSocket Client

A command-line client that runs on your local machine, connects to the WebSocket server (which can be running locally or in a container), and displays index updates.

```bash
# Run the client
cargo run --bin crypto-index-client

# Connect to a specific server
cargo run --bin crypto-index-client -- --server ws://example.com:9000

# Connect to a containerized collector
cargo run --bin crypto-index-client -- --server ws://localhost:9000
```

The client provides:
- Real-time display of index updates
- Automatic reconnection with exponential backoff
- Configurable connection settings

See [CLIENT.md](docs/CLIENT.md) for detailed documentation.

### 3. Supervisor

A process that monitors the main application, automatically restarts it on failure, and sends notifications.

```bash
# Run the supervisor
cargo run --bin crypto-index-supervisor

# Run with custom settings
cargo run --bin crypto-index-supervisor -- \
  --max-restarts 10 \
  --monitoring-period-minutes 30 \
  --notification-script ./notify.sh
```

The supervisor provides:
- Automatic restart on application failure
- Exponential backoff for restart attempts
- Configurable restart limits and monitoring periods
- Notification system for restart events

See [SUPERVISOR.md](docs/SUPERVISOR.md) for detailed documentation.

## Testing

The project includes comprehensive unit tests for critical components, particularly the smoothing algorithms:

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test --lib smoothing
```

The smoothing algorithm tests verify:
- None/passthru algorithm works correctly
- 20-point Simple Moving Average (SMA) calculates correctly
- 20-point Exponential Moving Average (EMA) follows the formula:
  ```
  EMA = P*a + EMA_prev*(1-a)
  where a = s/(1+N)
  ```

## Project Structure

The project follows a modular structure for maintainability and extensibility:

```
src/
├── bin/                    # Binary executables
│   ├── collector.rs        # Main application
│   ├── client.rs           # WebSocket client
│   └── supervisor.rs       # Supervisor process
├── config/                 # Configuration handling
├── exchange/               # Exchange integrations
├── index/                  # Index calculation
├── storage/                # Data persistence
├── smoothing/              # Smoothing algorithms
├── websocket/              # WebSocket server
├── notification/           # Notification system
├── logging/                # Logging utilities
└── lib.rs                  # Library exports
```

## Limitations and Future Work

- **Dynamic Configuration**: The current implementation uses static TOML configuration instead of etcd and doesn't support in-flight changes to indices or feeds.
- **Exchange Support**: Kraken exchange is not implemented.
- **Advanced Features**: Some "nice-to-have" features like configurable smoothing parameters and RPC endpoints are not implemented.

## License

This project is licensed under the MIT License - see the LICENSE file for details.