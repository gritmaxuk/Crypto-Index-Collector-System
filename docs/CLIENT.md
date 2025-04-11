# Crypto Index Client

The Crypto Index Client is a command-line WebSocket client that connects to the Crypto Index Collector's WebSocket server and displays real-time index updates.

## Features

- Connects to the WebSocket server and displays index updates in real-time
- Automatically reconnects if the connection is lost
- Uses exponential backoff for reconnection attempts
- Configurable server address and reconnection settings
- Formatted display of index updates

## Usage

```bash
# Run with default settings (connects to ws://127.0.0.1:9000)
cargo run --bin crypto-index-client

# Connect to a specific WebSocket server
cargo run --bin crypto-index-client -- --server ws://example.com:8080

# Disable automatic reconnection
cargo run --bin crypto-index-client -- --no-reconnect

# Set custom reconnection delay (in seconds)
cargo run --bin crypto-index-client -- --reconnect-delay 10
```

### Connecting to a Containerized Collector

If you're running the collector in a Docker container, you can connect to it from your host machine:

```bash
# Connect to the containerized collector (default port mapping)
cargo run --bin crypto-index-client -- --server ws://localhost:9000
```

Make sure the WebSocket port (default: 9000) is properly exposed in your Docker configuration.

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

Other messages from the server are displayed with the `[SERVER MESSAGE]` prefix.

## Reconnection Strategy

If the connection to the WebSocket server is lost, the client will automatically attempt to reconnect using an exponential backoff strategy:

1. First attempt: Wait for the configured reconnect delay (default: 5 seconds)
2. Subsequent attempts: Double the delay each time, up to a maximum of 60 seconds

This prevents overwhelming the server with reconnection attempts if it's experiencing issues.

## Logging

The client uses structured logging with clear prefixes:

- `[CLIENT]`: General client messages
- `[INDEX UPDATE]`: Index updates received from the server
- `[SERVER MESSAGE]`: Other messages received from the server
- `[CONNECTION]`: Connection status messages

Example log output:

```
[2023-05-01T12:34:56Z INFO  crypto_index_client] [CLIENT] Crypto Index Client starting up
[2023-05-01T12:34:56Z INFO  crypto_index_client] [CLIENT] Connecting to WebSocket server at ws://127.0.0.1:9000
[2023-05-01T12:34:56Z INFO  crypto_index_client] [CLIENT] Connected to the server successfully
[2023-05-01T12:34:56Z INFO  crypto_index_client] [SERVER MESSAGE] Connected to Crypto Index Collector. Client: 127.0.0.1:54321
[2023-05-01T12:34:57Z INFO  crypto_index_client] [INDEX UPDATE] BTC-USD-INDEX = 42000.50 (2023-05-01T12:34:57Z)
```

## Error Handling

The client implements robust error handling:

- Gracefully handles connection failures
- Automatically reconnects with exponential backoff
- Provides clear error messages
- Continues operation even after temporary connection issues

## Integration with Other Tools

The client's output can be easily piped to other tools for further processing:

```bash
# Save index updates to a file
cargo run --bin crypto-index-client | grep "INDEX UPDATE" > index_updates.log

# Process index updates with another tool
cargo run --bin crypto-index-client | your-processing-tool
```
