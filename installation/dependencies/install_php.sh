#!/bin/bash

enable_php_modules() {
    log_step "8/8 - Enabling Apache mod_phpX.Y for installed PHP versions"

    local found=0
    local enabled=0
    local ver mod_name mod_load enabled_link conf_src conf_link

    for ver in 5.6 7.4 8.0 8.1 8.2 8.3 8.4; do
        if [ "$ver" = "5.6" ]; then
            if [ -f "/etc/apache2/mods-available/php5.6.load" ]; then
                mod_name="php5.6"
            elif [ -f "/etc/apache2/mods-available/php5.load" ]; then
                mod_name="php5"
            else
                log_info "PHP 5.6 has no Apache module"
                continue
            fi
        else
            mod_name="php${ver}"
        fi

        mod_load="/etc/apache2/mods-available/${mod_name}.load"
        [ ! -f "$mod_load" ] && log_info "PHP $ver module not found" && continue
        found=$((found + 1))

        if command -v a2enmod >/dev/null 2>&1; then
            run_cmd "a2enmod $mod_name" a2enmod "$mod_name" \
                && log_ok "Enabled mod_${mod_name}" \
                || log_info "mod_${mod_name} already enabled or skipped"
            enabled=$((enabled + 1))
            continue
        fi

        enabled_link="/etc/apache2/mods-enabled/${mod_name}.load"
        conf_src="/etc/apache2/mods-available/${mod_name}.conf"
        conf_link="/etc/apache2/mods-enabled/${mod_name}.conf"

        [ ! -L "$enabled_link" ] && run_cmd "symlink $mod_name.load" ln -s "$mod_load" "$enabled_link" || true
        [ -f "$conf_src" ] && [ ! -L "$conf_link" ] && run_cmd "symlink $mod_name.conf" ln -s "$conf_src" "$conf_link" || true
        enabled=$((enabled + 1))
    done

    if [ "$found" -eq 0 ]; then
        log_warn "No mod_phpX.Y found"
        log_warn "Install PHP: apt-get install -y libapache2-mod-php8.2"
    else
        log_ok "PHP Apache mods: $enabled/$found version(s) enabled"
    fi
}
