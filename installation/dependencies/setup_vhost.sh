#!/bin/bash

create_devpanel_conf() {
    log_step "5/8 - Creating devpanel.conf"
    if [ -f "$DEVPANEL_CONF" ]; then
        log_info "Already exists: $DEVPANEL_CONF"
        return 0
    fi

    cat > "$DEVPANEL_CONF" << 'APACHECONF'
# DevPanel managed VirtualHosts
# Managed by DevPanel - use the GUI to add/edit/remove entries below.
# Each <VirtualHost> block is one .local project.

APACHECONF
    log_ok "Created: $DEVPANEL_CONF"
}

enable_devpanel_site() {
    log_step "6/8 - Enabling devpanel.conf"
    if command_exists a2ensite; then
        run_cmd "a2ensite devpanel.conf" a2ensite devpanel.conf \
            && log_ok "a2ensite devpanel.conf succeeded" \
            || log_info "devpanel.conf may already be enabled"
        return 0
    fi

    local enabled_link="$DEV_APACHE_SITES_ENABLED/devpanel.conf"
    if [ ! -L "$enabled_link" ]; then
        run_cmd "symlink devpanel.conf" ln -s "$DEVPANEL_CONF" "$enabled_link" \
            && log_ok "Created symlink: $enabled_link" \
            || log_err "Failed to create symlink: $enabled_link"
    else
        log_info "Symlink already exists: $enabled_link"
    fi
}
