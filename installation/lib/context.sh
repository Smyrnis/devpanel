#!/bin/bash

require_root() {
    if [ "$EUID" -ne 0 ]; then
        log_err "Not running as root"
        exit 1
    fi
}

detect_target_user() {
    if [ -z "$SUDO_USER" ]; then
        log_err "Cannot detect target user (SUDO_USER is empty)"
        exit 1
    fi

    REAL_USER="$SUDO_USER"
    USER_HOME=$(eval echo "~$SUDO_USER")
    PROJECTS_DIR="$USER_HOME/projects"
    WEBROOT="$DEV_WEBROOT"
    DEVPANEL_CONF="$DEV_VHOST_CONF"
    DEFAULT_SITE="$DEV_DEFAULT_SITE"
    CFG_DIR="$(devpanel_config_dir_for_home "$USER_HOME")"
    CFG_FILE="$(devpanel_config_file_for_home "$USER_HOME")"
}

resolve_index_php() {
    local root_dir="$1"
    INDEX_PHP_SRC="$root_dir/share/index.php"
    [ ! -f "$INDEX_PHP_SRC" ] && INDEX_PHP_SRC="$DEV_SHARE_DIR/index.php"
}

log_context() {
    log_info "User:     $REAL_USER"
    log_info "Home:     $USER_HOME"
    log_info "Projects: $PROJECTS_DIR"
    log_info "Webroot:  $WEBROOT"
    log_info "VHosts:   $DEVPANEL_CONF"
}
