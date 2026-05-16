#!/bin/bash

check_mysql() {
    log_info "Checking MySQL/MariaDB availability"
    if command -v mysql >/dev/null 2>&1; then
        log_ok "mysql client found: $(command -v mysql)"
    elif command -v mariadb >/dev/null 2>&1; then
        log_ok "mariadb client found: $(command -v mariadb)"
    else
        log_warn "No MySQL/MariaDB client found"
    fi

    if systemctl list-unit-files mysql.service >/dev/null 2>&1 || systemctl list-unit-files mariadb.service >/dev/null 2>&1; then
        log_ok "Database service unit found"
    else
        log_warn "No mysql.service or mariadb.service unit found"
    fi
}
