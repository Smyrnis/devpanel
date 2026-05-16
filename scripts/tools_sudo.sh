#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root

case "${1:-}" in
    apt-install)
        require_arg "${2:-}" "package"
        apt-get install -y "$2"
        ;;
    apt-remove)
        require_arg "${2:-}" "package"
        apt-get remove -y "$2"
        ;;
    composer-install)
        php -r "copy('https://getcomposer.org/installer', '/tmp/composer-setup.php');"
        php /tmp/composer-setup.php --install-dir=/usr/local/bin --filename=composer
        rm -f /tmp/composer-setup.php
        ;;
    composer-update)
        composer self-update
        ;;
    redis-start)
        run_systemctl start redis-server
        ;;
    redis-stop)
        run_systemctl stop redis-server
        ;;
    *)
        echo "Usage: $0 {apt-install PKG|apt-remove PKG|composer-install|composer-update|redis-start|redis-stop}" >&2
        exit 2
        ;;
esac
