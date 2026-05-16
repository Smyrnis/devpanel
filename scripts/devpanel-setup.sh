#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -x "$ROOT_DIR/installation/devpanel-setup.sh" ]; then
    exec "$ROOT_DIR/installation/devpanel-setup.sh" "$@"
fi

if [ -x "/usr/share/devpanel/installation/devpanel-setup.sh" ]; then
    exec "/usr/share/devpanel/installation/devpanel-setup.sh" "$@"
fi

LOG_DIR="/var/log/devpanel"
LOG_FILE="$LOG_DIR/setup.log"
mkdir -p "$LOG_DIR"
echo "$(date '+%Y-%m-%d %H:%M:%S') [ERROR] installation/devpanel-setup.sh not found" >> "$LOG_FILE"
exit 1
