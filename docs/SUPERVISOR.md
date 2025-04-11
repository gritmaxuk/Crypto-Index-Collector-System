# Crypto Index Supervisor

The Crypto Index Supervisor is a process that monitors the Crypto Index Collector, automatically restarts it on failure, and sends notifications when restarts occur.

## Features

- Monitors the main application and detects crashes
- Automatically restarts the application when it fails
- Uses exponential backoff for restart attempts
- Sends notifications on restarts
- Configurable restart limits and monitoring periods
- Prevents restart loops by giving up after too many failures

## Usage

```bash
# Start the supervisor with default settings
cargo run --bin crypto-index-supervisor

# Start with custom settings
cargo run --bin crypto-index-supervisor -- \
  --max-restarts 10 \
  --monitoring-period-minutes 30 \
  --initial-restart-delay 10 \
  --max-restart-delay 120 \
  --notification-script ./notify.sh
```

## Command-Line Options

```
Usage: crypto-index-supervisor [OPTIONS]

Options:
      --max-restarts <MAX_RESTARTS>
          Maximum number of restarts in the monitoring period before giving up [default: 5]
          
      --monitoring-period-minutes <MONITORING_PERIOD_MINUTES>
          Monitoring period in minutes [default: 10]
          
      --initial-restart-delay <INITIAL_RESTART_DELAY>
          Initial delay before restarting after a failure (in seconds) [default: 5]
          
      --max-restart-delay <MAX_RESTART_DELAY>
          Maximum delay between restarts (in seconds) [default: 60]
          
      --notification-script <NOTIFICATION_SCRIPT>
          Path to the notification script (if any)
          
  -h, --help
          Print help
          
  -V, --version
          Print version
```

## Notification System

The supervisor can send notifications when the application crashes and is restarted. To enable notifications, provide a path to a notification script using the `--notification-script` option.

A sample notification script (`notify.sh`) is included in the repository. This script:

1. Logs all notifications to a file (`notifications.log`)
2. Prints notifications to the console
3. Can be extended to send notifications via email, Slack, Discord, etc.

### Customizing Notifications

Edit the `notify.sh` script to enable additional notification methods:

```bash
#!/bin/bash
# Simple notification script for Crypto Index Collector

# Configuration
LOG_FILE="notifications.log"
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
MESSAGE="$1"

# Log the notification
echo "[$TIMESTAMP] $MESSAGE" >> "$LOG_FILE"

# Email notification (requires mailx)
# echo "$MESSAGE" | mail -s "Crypto Index Collector Alert" your-email@example.com

# Slack notification (requires curl)
# SLACK_WEBHOOK_URL="your-webhook-url"
# curl -X POST -H 'Content-type: application/json' --data "{\"text\":\"$MESSAGE\"}" "$SLACK_WEBHOOK_URL"

# Discord notification (requires curl)
# DISCORD_WEBHOOK_URL="your-webhook-url"
# curl -X POST -H "Content-Type: application/json" --data "{\"content\":\"$MESSAGE\"}" "$DISCORD_WEBHOOK_URL"

# Print to console as well
echo "[$TIMESTAMP] $MESSAGE"

# Exit successfully
exit 0
```

## Restart Strategy

The supervisor uses an exponential backoff strategy for restart attempts:

1. First restart: Wait for the initial restart delay (default: 5 seconds)
2. Subsequent restarts: Double the delay each time, up to the maximum delay (default: 60 seconds)

This prevents overwhelming the system with rapid restart attempts if there's a persistent issue.

## Monitoring Period

The supervisor tracks the number of restarts within a monitoring period (default: 10 minutes). If the application is restarted too many times within this period, the supervisor will give up to prevent an infinite restart loop.

The restart counter is reset after the monitoring period elapses.

## Logging

The supervisor uses structured logging with clear prefixes:

- `[SUPERVISOR]`: General supervisor messages
- `[NOTIFICATION]`: Notification messages
- `[WARNING]`: Warning messages
- `[ERROR]`: Error messages

Example log output:

```
[2023-05-01T12:34:56Z INFO  crypto_index_supervisor] [SUPERVISOR] Starting Crypto Index Collector supervisor
[2023-05-01T12:34:56Z INFO  crypto_index_supervisor] [SUPERVISOR] Starting Crypto Index Collector
[2023-05-01T12:35:56Z WARN  crypto_index_supervisor] [SUPERVISOR] Crypto Index Collector failed with exit code: 1
[2023-05-01T12:35:56Z INFO  crypto_index_supervisor] [NOTIFICATION] WARNING: Crypto Index Collector crashed with exit code 1. Restarting in 5 seconds (attempt 1/5)
[2023-05-01T12:35:56Z INFO  crypto_index_supervisor] [SUPERVISOR] Restarting in 5 seconds (attempt 1/5)
```

## Production Deployment

For production environments, consider:

1. Running the supervisor as a systemd service
2. Setting up more robust notification methods (email, Slack, PagerDuty, etc.)
3. Configuring appropriate restart limits and delays based on your application's characteristics

### Example systemd Service

```ini
[Unit]
Description=Crypto Index Collector Supervisor
After=network.target

[Service]
User=crypto
WorkingDirectory=/opt/crypto-index-collector
ExecStart=/opt/crypto-index-collector/target/release/crypto-index-supervisor --notification-script /opt/crypto-index-collector/notify.sh
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```
