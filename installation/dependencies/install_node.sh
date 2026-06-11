#!/bin/bash

NVM_INSTALL_VERSION="${NVM_INSTALL_VERSION:-v0.40.1}"

install_node_nvm_if_requested() {
    if [ "${DEVPANEL_INSTALL_NODE_NVM:-0}" != "1" ]; then
        log_info "Node/NVM install skipped"
        return 0
    fi

    local version="${DEVPANEL_NODE_VERSION:-22}"
    install_node_nvm_version "$version"
}

install_node_nvm_version() {
    local version="${1:-22}"
    local version_arg
    local installer_url
    local installer_url_arg
    log_step "Installing NVM and Node $version"

    if [ -z "${REAL_USER:-}" ] || [ -z "${USER_HOME:-}" ]; then
        log_err "Cannot install NVM without REAL_USER and USER_HOME"
        return 1
    fi

    run_cmd "install NVM prerequisites" apt-get install -y curl ca-certificates \
        || return 1

    printf -v version_arg '%q' "$version"
    installer_url="https://raw.githubusercontent.com/nvm-sh/nvm/$NVM_INSTALL_VERSION/install.sh"
    printf -v installer_url_arg '%q' "$installer_url"

    local script
    script="set -e -o pipefail"
    script="$script; export NVM_DIR=\"\$HOME/.nvm\""
    script="$script; if [ ! -s \"\$NVM_DIR/nvm.sh\" ]; then curl -o- $installer_url_arg | bash; fi"
    script="$script; . \"\$NVM_DIR/nvm.sh\""
    script="$script; nvm install $version_arg"
    script="$script; nvm alias default $version_arg"
    script="$script; nvm use $version_arg"

    run_cmd "install Node $version with NVM" sudo -u "$REAL_USER" env HOME="$USER_HOME" bash -lc "$script" \
        || return 1

    log_ok "Node $version ready through NVM"
}
