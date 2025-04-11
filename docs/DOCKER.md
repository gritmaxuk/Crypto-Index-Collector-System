# Docker Deployment

The Crypto Index Collector can be deployed using Docker and Docker Compose for easy setup and management.

## Components

The Docker deployment includes:

1. **TimescaleDB**: A PostgreSQL database with the TimescaleDB extension for time-series data storage
2. **Crypto Index Collector**: The main application, supervised by the supervisor process

## Prerequisites

- Docker and Docker Compose installed on your system
- Basic understanding of Docker and containerization

## Quick Start

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

## Configuration

The Docker deployment uses the `config.toml` file from the host system, mounted into the container. You can modify this file to change the configuration without rebuilding the container.

Make sure to update the database URL in the configuration to use the Docker service name:

```toml
[database]
enabled = true
url = "postgres://postgres:password@timescaledb:5432/crypto_indices"
retention_days = 30
```

## Dockerfile Explanation

The `Dockerfile` uses a multi-stage build process:

1. **Builder Stage**:
   - Uses the official Rust image as the base
   - Builds the application in release mode
   - Optimizes dependency caching for faster builds

2. **Runtime Stage**:
   - Uses a minimal Debian image
   - Installs only the necessary runtime dependencies
   - Copies the compiled binaries from the builder stage
   - Sets up the notification script
   - Uses the supervisor as the entrypoint

## Docker Compose Explanation

The `docker-compose.yml` file defines two services:

1. **timescaledb**:
   - Uses the official TimescaleDB image
   - Sets up the database credentials
   - Persists data using a Docker volume
   - Exposes port 5432 for database connections
   - Includes a healthcheck to ensure the database is ready before starting the collector

2. **collector**:
   - Builds the application using the Dockerfile
   - Depends on the TimescaleDB service
   - Mounts the configuration file from the host
   - Exposes port 9000 for WebSocket connections
   - Configures automatic restart

## Volumes

The Docker Compose setup uses a named volume for the TimescaleDB data:

- `timescaledb_data`: Stores the PostgreSQL data files

This ensures that your data persists even if the containers are removed.

## Accessing the Application

Once the containers are running, you can connect to the WebSocket server using the client application from your host machine:

```bash
# Build the client
cargo build --bin crypto-index-client

# Run the client to connect to the containerized collector
cargo run --bin crypto-index-client -- --server ws://localhost:9000
```

### Client-Server Architecture

The Crypto Index Collector system uses a client-server architecture:

- **Server (Containerized)**: The collector and supervisor run inside Docker containers
- **Client (Host Machine)**: The client application runs on your local machine and connects to the containerized server

This separation allows multiple clients to connect to a single collector instance and provides flexibility in deployment.

## Production Considerations

For production deployments, consider:

1. **Security**:
   - Use environment variables for sensitive information
   - Set up proper network isolation
   - Use Docker secrets for credentials

2. **Monitoring**:
   - Set up container monitoring (e.g., Prometheus, Grafana)
   - Configure log aggregation (e.g., ELK stack)

3. **High Availability**:
   - Use Docker Swarm or Kubernetes for orchestration
   - Set up database replication
   - Configure proper resource limits

4. **Backup**:
   - Set up regular database backups
   - Document the restore process
