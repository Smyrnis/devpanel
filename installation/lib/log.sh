#!/bin/bash

LOG_DIR="${LOG_DIR:-/var/log/devpanel}"
LOG_FILE="${LOG_FILE:-$LOG_DIR/setup.log}"

log_init() {
    mkdir -p "$LOG_DIR"
    chmod 755 "$LOG_DIR"
    : > "$LOG_FILE"
    chmod 644 "$LOG_FILE"
}

log_ts() { date '+%Y-%m-%d %H:%M:%S'; }

log_write() {
    local level="$1"
    shift
    printf '%s [%-5s] %s\n' "$(log_ts)" "$level" "$*" >> "$LOG_FILE"
}

log_step() { log_write "STEP" "$*"; }
log_info() { log_write "INFO" "$*"; }
log_ok() { log_write "OK" "$*"; }
log_warn() { log_write "WARN" "$*"; }
log_err() { log_write "ERROR" "$*"; }
log_cmd() { log_write "CMD" "$*"; }
log_out() { log_write "OUT" "$*"; }
