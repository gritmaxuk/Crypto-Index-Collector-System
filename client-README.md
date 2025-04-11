# Crypto Index Client

A command-line WebSocket client for receiving real-time cryptocurrency index updates from the Crypto Index Collector.

## Features

- Connects to the Crypto Index Collector WebSocket server
- Displays real-time index updates in a formatted way
- Automatically reconnects if the connection is lost
- Configurable server address and reconnection settings

## Usage

```bash
# Connect to the default WebSocket server (ws://127.0.0.1:9000)
cargo run --bin crypto-index-client

# Connect to a specific WebSocket server
cargo run --bin crypto-index-client -- --server ws://example.com:8080

# Disable automatic reconnection
cargo run --bin crypto-index-client -- --no-reconnect

# Set custom reconnection delay (in seconds)
cargo run --bin crypto-index-client -- --reconnect-delay 10
```

## Command-Line Options

```
Usage: crypto-index-client [OPTIONS]

Options:
  -s, --server <SERVER>              WebSocket server address [default: ws://127.0.0.1:9000]
  -r, --reconnect                    Reconnect automatically if connection is lost [default: true]
      --reconnect-delay <RECONNECT_DELAY>  Reconnection delay in seconds [default: 5]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Output Format

The client displays index updates in the following format:

```
[INDEX UPDATE] BTC-USD-INDEX = 42000.25 (2023-05-01T12:34:56.789Z)
```

This shows:
- The index name (BTC-USD-INDEX)
- The current index value (42000.25)
- The timestamp of the update (2023-05-01T12:34:56.789Z)

## Reconnection Strategy

If the connection to the WebSocket server is lost, the client will automatically attempt to reconnect using an exponential backoff strategy:

1. First attempt: Wait for the configured reconnect delay (default: 5 seconds)
2. Subsequent attempts: Double the delay each time, up to a maximum of 60 seconds

This prevents overwhelming the server with reconnection attempts if it's experiencing issues.
