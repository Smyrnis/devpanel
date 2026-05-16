#!/bin/bash

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

run_cmd() {
    local description="$1"
    shift
    log_cmd "$description: $*"

    local output
    output=$("$@" 2>&1)
    local rc=$?

    if [ -n "$output" ]; then
        while IFS= read -r line; do
            log_out "$line"
        done <<< "$output"
    fi

    return "$rc"
}

safe_chown() {
    chown "$@" 2>>"$LOG_FILE" || true
}

safe_chmod() {
    chmod "$@" 2>>"$LOG_FILE" || true
}

write_file() {
    local path="$1"
    local owner="${2:-}"
    local mode="${3:-}"

    cat > "$path"

    [ -n "$owner" ] && safe_chown "$owner" "$path"
    [ -n "$mode" ] && safe_chmod "$mode" "$path"
}
