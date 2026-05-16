#!/bin/bash

DEV_HOSTS_FILE="/etc/hosts"
DEV_COMPOSER_INSTALL_DIR="/usr/local/bin"

apache_service_name() {
    printf 'apache2'
}

mysql_service_name() {
    printf '%s' "${MYSQL_SERVICE:-mysql}"
}

redis_service_name() {
    if systemctl list-unit-files redis-server.service >/dev/null 2>&1; then
        printf 'redis-server'
    else
        printf 'redis'
    fi
}
