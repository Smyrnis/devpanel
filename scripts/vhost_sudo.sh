#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root

case "${1:-}" in
    write-conf)
        require_arg "${2:-}" "path"
        cat > "$2"
        systemctl reload apache2
        ;;
    append-host)
        require_arg "${2:-}" "hostname"
        if ! grep -q "[[:space:]]$2\\b" /etc/hosts; then
            printf '127.0.0.1    %s\n' "$2" >> /etc/hosts
        fi
        ;;
    reload)
        systemctl reload apache2
        ;;
    *)
        echo "Usage: $0 {write-conf PATH|append-host HOSTNAME|reload}" >&2
        exit 2
        ;;
esac
