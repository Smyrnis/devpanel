#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root

SERVICE="$(mysql_service_name)"

case "${1:-}" in
    start|stop|restart|reload|status)
        run_systemctl "$1" "$SERVICE"
        ;;
    *)
        usage_error "$0 {start|stop|restart|reload|status}"
        ;;
esac
