# Build stage
FROM rust:1.71-slim-bullseye as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary and configuration
COPY --from=builder /app/target/release/crypto-index-collector /app/
COPY config.toml /app/

# Set the entrypoint
ENTRYPOINT ["/app/crypto-index-collector"]