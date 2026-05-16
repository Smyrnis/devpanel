#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root
APACHE_SERVICE="$(apache_service_name)"

case "${1:-}" in
    start|stop|restart|reload|status)
        run_systemctl "$1" "$APACHE_SERVICE"
        ;;
    enable-site)
        require_arg "${2:-}" "site"
        a2ensite "$2"
        systemctl reload "$APACHE_SERVICE"
        ;;
    disable-site)
        require_arg "${2:-}" "site"
        a2dissite "$2"
        systemctl reload "$APACHE_SERVICE"
        ;;
    enable-mod)
        require_arg "${2:-}" "module"
        a2enmod "$2"
        systemctl reload "$APACHE_SERVICE"
        ;;
    disable-mod)
        require_arg "${2:-}" "module"
        a2dismod "$2"
        systemctl reload "$APACHE_SERVICE"
        ;;
    configtest)
        apache2ctl configtest
        ;;
    *)
        usage_error "$0 {start|stop|restart|reload|status|enable-site SITE|disable-site SITE|enable-mod MOD|disable-mod MOD|configtest}"
        ;;
esac
