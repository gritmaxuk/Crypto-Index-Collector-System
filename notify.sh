#!/bin/bash
# Simple notification script for Crypto Index Collector
# This script logs notifications to a file and can be extended to send emails, Slack messages, etc.

# Configuration
LOG_FILE="notifications.log"
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
MESSAGE="$1"

# Log the notification
echo "[$TIMESTAMP] $MESSAGE" >> "$LOG_FILE"

# Print to console as well
echo "[$TIMESTAMP] $MESSAGE"

# Exit successfully
exit 0
