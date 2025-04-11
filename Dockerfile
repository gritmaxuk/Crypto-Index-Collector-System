FROM rust:1.70 as builder

WORKDIR /app

# Copy the Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/collector.rs && \
    echo "fn main() {}" > src/bin/client.rs && \
    echo "fn main() {}" > src/bin/supervisor.rs && \
    echo "pub fn main() {}" > src/lib.rs && \
    cargo build --release

# Copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bullseye-slim

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the binaries from the builder stage
COPY --from=builder /app/target/release/crypto-index-collector /usr/local/bin/
COPY --from=builder /app/target/release/crypto-index-supervisor /usr/local/bin/

# Copy the notification script
COPY notify.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/notify.sh

# Create a directory for configuration
WORKDIR /app
COPY config.toml .

# Set the entrypoint to the supervisor
ENTRYPOINT ["/usr/local/bin/crypto-index-supervisor", "--notification-script", "/usr/local/bin/notify.sh"]

# Expose the WebSocket port
EXPOSE 9000
