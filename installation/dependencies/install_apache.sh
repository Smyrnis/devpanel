#!/bin/bash

install_apache_check() {
    log_step "1/8 - Checking Apache installation"
    if ! command_exists apache2 && ! command_exists apachectl; then
        log_err "Apache2 is not installed"
        log_err "Install it first: apt-get install -y apache2 libapache2-mod-php"
        exit 1
    fi
    log_ok "Apache2 found: $(command -v apache2 2>/dev/null || command -v apachectl)"
}

configure_default_site() {
    log_step "4/8 - Setting DirectoryIndex in 000-default.conf"
    if [ ! -f "$DEFAULT_SITE" ]; then
        log_warn "000-default.conf not found at $DEFAULT_SITE"
        return 0
    fi

    if [ ! -f "${DEFAULT_SITE}.devpanel.bak" ]; then
        run_cmd "backup 000-default.conf" cp "$DEFAULT_SITE" "${DEFAULT_SITE}.devpanel.bak" \
            && log_ok "Backup saved: ${DEFAULT_SITE}.devpanel.bak" \
            || log_warn "Backup failed"
    else
        log_info "Backup already exists"
    fi

    if grep -q "DirectoryIndex" "$DEFAULT_SITE"; then
        run_cmd "update DirectoryIndex" \
            sed -i 's|^\s*DirectoryIndex .*|    DirectoryIndex index.php index.html index.htm|' "$DEFAULT_SITE" \
            && log_ok "Updated DirectoryIndex" \
            || log_warn "Failed to update DirectoryIndex"
    else
        run_cmd "insert DirectoryIndex" \
            sed -i '/<VirtualHost \*:80>/a\    DirectoryIndex index.php index.html index.htm' "$DEFAULT_SITE" \
            && log_ok "Inserted DirectoryIndex" \
            || log_warn "Failed to insert DirectoryIndex"
    fi
}

enable_rewrite() {
    log_step "7/8 - Enabling mod_rewrite"
    if command_exists a2enmod; then
        run_cmd "a2enmod rewrite" a2enmod rewrite \
            && log_ok "mod_rewrite enabled" \
            || log_info "mod_rewrite already enabled or unavailable"
    else
        log_warn "a2enmod not found"
    fi
}

reload_or_start_apache() {
    log_info "Testing Apache configuration"
    if ! run_cmd "apache2ctl configtest" apache2ctl configtest; then
        log_warn "Apache config test failed"
        return 0
    fi

    log_ok "Apache configuration OK"
    if systemctl is-active --quiet apache2; then
        run_cmd "systemctl reload apache2" systemctl reload apache2 \
            && log_ok "Apache reloaded" \
            || log_warn "Apache reload failed"
    else
        run_cmd "systemctl start apache2" systemctl start apache2 \
            && log_ok "Apache started" \
            || log_warn "Apache start failed"
    fi
}
