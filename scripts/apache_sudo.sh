#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root

case "${1:-}" in
    start|stop|restart|reload|status)
        run_systemctl "$1" apache2
        ;;
    enable-site)
        require_arg "${2:-}" "site"
        a2ensite "$2"
        systemctl reload apache2
        ;;
    disable-site)
        require_arg "${2:-}" "site"
        a2dissite "$2"
        systemctl reload apache2
        ;;
    enable-mod)
        require_arg "${2:-}" "module"
        a2enmod "$2"
        systemctl reload apache2
        ;;
    disable-mod)
        require_arg "${2:-}" "module"
        a2dismod "$2"
        systemctl reload apache2
        ;;
    configtest)
        apache2ctl configtest
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|reload|status|enable-site SITE|disable-site SITE|enable-mod MOD|disable-mod MOD|configtest}" >&2
        exit 2
        ;;
esac
