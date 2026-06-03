#!/bin/bash

enable_php_modules() {
    log_step "8/8 - Enabling Apache PHP-FPM support for installed PHP versions"

    local found=0
    local enabled=0
    local ver fpm_conf conf_src service

    if command_exists a2enmod; then
        run_cmd "a2enmod proxy_fcgi" a2enmod proxy_fcgi \
            && log_ok "proxy_fcgi enabled" \
            || log_info "proxy_fcgi already enabled or unavailable"
        run_cmd "a2enmod setenvif" a2enmod setenvif \
            && log_ok "setenvif enabled" \
            || log_info "setenvif already enabled or unavailable"
    else
        log_warn "a2enmod not found"
    fi

    for ver in 5.6 7.0 7.1 7.2 7.3 7.4 8.0 8.1 8.2 8.3 8.4 8.5; do
        fpm_conf="php${ver}-fpm"
        conf_src="$DEV_APACHE_CONF_AVAILABLE/${fpm_conf}.conf"
        service="php${ver}-fpm"

        [ ! -f "$conf_src" ] && log_info "PHP $ver FPM Apache config not found" && continue
        found=$((found + 1))

        if command_exists a2enconf; then
            run_cmd "a2enconf $fpm_conf" a2enconf "$fpm_conf" \
                && log_ok "Enabled ${fpm_conf}" \
                || log_info "${fpm_conf} already enabled or skipped"
        fi

        if systemctl list-unit-files "${service}.service" >/dev/null 2>&1; then
            run_cmd "systemctl enable --now $service" systemctl enable --now "$service" \
                && log_ok "Started ${service}" \
                || log_warn "Could not start ${service}"
        else
            log_info "${service}.service not found"
        fi
        enabled=$((enabled + 1))
    done

    if [ "$found" -eq 0 ]; then
        log_warn "No PHP-FPM Apache configs found"
        log_warn "Install PHP-FPM: apt-get install -y php8.5-fpm"
    else
        log_ok "PHP-FPM configs: $enabled/$found version(s) enabled"
    fi
}
