#!/bin/bash

set -e

require_root() {
    if [ "$EUID" -ne 0 ]; then
        echo "This command must run as root" >&2
        exit 1
    fi
}

require_arg() {
    local value="$1"
    local name="$2"
    if [ -z "$value" ]; then
        echo "Missing argument: $name" >&2
        exit 2
    fi
}

run_systemctl() {
    local action="$1"
    local service="$2"
    require_arg "$action" "action"
    require_arg "$service" "service"
    systemctl "$action" "$service"
}
