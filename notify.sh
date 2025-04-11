#!/bin/bash
# Simple notification script for Crypto Index Collector
# This script logs notifications to a file and can be extended to send emails, Slack messages, etc.

# Configuration
LOG_FILE="notifications.log"
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
MESSAGE="$1"

# Log the notification
echo "[$TIMESTAMP] $MESSAGE" >> "$LOG_FILE"

# Uncomment and configure one of the following notification methods:

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
