#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root

SERVICE="${MYSQL_SERVICE:-mysql}"

case "${1:-}" in
    start|stop|restart|reload|status)
        run_systemctl "$1" "$SERVICE"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|reload|status}" >&2
        exit 2
        ;;
esac
