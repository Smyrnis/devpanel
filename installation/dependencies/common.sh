#!/bin/bash

set -e

LOG_DIR="${LOG_DIR:-/var/log/devpanel}"
LOG_FILE="${LOG_FILE:-$LOG_DIR/setup.log}"

init_log() {
    mkdir -p "$LOG_DIR"
    chmod 755 "$LOG_DIR"
    : > "$LOG_FILE"
    chmod 644 "$LOG_FILE"
}

ts() { date '+%Y-%m-%d %H:%M:%S'; }

log_step() { echo "$(ts) [STEP]  $1" >> "$LOG_FILE"; }
log_info() { echo "$(ts) [INFO]  $1" >> "$LOG_FILE"; }
log_ok() { echo "$(ts) [OK]    $1" >> "$LOG_FILE"; }
log_warn() { echo "$(ts) [WARN]  $1" >> "$LOG_FILE"; }
log_err() { echo "$(ts) [ERROR] $1" >> "$LOG_FILE"; }
log_cmd() { echo "$(ts) [CMD]   $1" >> "$LOG_FILE"; }

run_cmd() {
    local description="$1"
    shift
    log_cmd "$description: $*"
    local output
    output=$("$@" 2>&1)
    local rc=$?
    if [ -n "$output" ]; then
        while IFS= read -r line; do
            echo "$(ts) [OUT]   $line" >> "$LOG_FILE"
        done <<< "$output"
    fi
    return "$rc"
}

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
    WEBROOT="/var/www/html"
    DEVPANEL_CONF="/etc/apache2/sites-available/devpanel.conf"
    DEFAULT_SITE="/etc/apache2/sites-available/000-default.conf"
    CFG_DIR="$USER_HOME/.config/devpanel"
    CFG_FILE="$CFG_DIR/config.toml"
}

resolve_index_php() {
    local root_dir="$1"
    INDEX_PHP_SRC="$root_dir/share/index.php"
    [ ! -f "$INDEX_PHP_SRC" ] && INDEX_PHP_SRC="/usr/share/devpanel/index.php"
}
