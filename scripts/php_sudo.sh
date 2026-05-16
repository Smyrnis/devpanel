#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/common_sudo.sh"

require_root

case "${1:-}" in
    switch)
        require_arg "${2:-}" "php binary"
        update-alternatives --set php "$2"
        ;;
    install-version)
        require_arg "${2:-}" "version"
        apt-get update
        apt-get install -y "php$2" "php$2-cli" "php$2-common" "php$2-mysql" "php$2-xml" "php$2-mbstring"
        ;;
    remove-version)
        require_arg "${2:-}" "version"
        apt-get remove -y "php$2" "php$2-*"
        ;;
    install-extension)
        require_arg "${2:-}" "package"
        apt-get install -y "$2"
        ;;
    remove-extension)
        require_arg "${2:-}" "package"
        apt-get remove -y "$2"
        ;;
    *)
        usage_error "$0 {switch PHP_BIN|install-version VERSION|remove-version VERSION|install-extension PKG|remove-extension PKG}"
        ;;
esac
