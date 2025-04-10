# Scope Adjustments and Trade-offs
This MVP implementation includes some adjustments from the original requirements:

1. Static vs. Dynamic Configuration: Using a static file configuration instead of etcd for simplicity and reduced dependencies.
2. Exchange Support: Implemented support for Coinbase and Binance. Kraken support could be added following the same pattern.
3. Notification System: Notifications are implemented via structured logging instead of a separate notification service.
4. Database Persistence: Raw price data is not persisted to a database in this MVP.
5. Observability: Basic logging is implemented instead of a full monitoring stack (Prometheus, OpenTelemetry).
6. Output Method: Index data is output to standard output (logs) rather than to specific endpoints or RPCs.

# Future Enhancements
The following enhancements could be implemented to extend the functionality:

1. Dynamic Configuration: Integrate with etcd to allow for dynamic configuration updates without restarting the service.
2. Additional Exchanges: Add support for Kraken and other exchanges.
3. Enhanced Observability: Integrate with Prometheus for metrics and OpenTelemetry for distributed tracing.
4. Database Integration: Add persistence layer to store historical price and index data.
3.  API Endpoints: Create REST API or gRPC endpoints to expose the index data.