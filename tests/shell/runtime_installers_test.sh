#!/bin/bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

declare -a CALLS=()
declare -a LOGS=()
RUN_CMD_FAILURE=""

log_step() { LOGS+=("STEP|$*"); }
log_info() { LOGS+=("INFO|$*"); }
log_ok() { LOGS+=("OK|$*"); }
log_err() { LOGS+=("ERROR|$*"); }

run_cmd() {
    local description="$1"
    shift
    CALLS+=("$description|$*")
    if [ "$RUN_CMD_FAILURE" = "$description" ]; then
        return 1
    fi
}

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_eq() {
    local expected="$1"
    local actual="$2"
    local label="$3"
    [ "$actual" = "$expected" ] || fail "$label: expected '$expected', got '$actual'"
}

assert_contains() {
    local value="$1"
    local expected="$2"
    local label="$3"
    [[ "$value" == *"$expected"* ]] || fail "$label: missing '$expected' in '$value'"
}

assert_not_contains() {
    local value="$1"
    local unexpected="$2"
    local label="$3"
    [[ "$value" != *"$unexpected"* ]] || fail "$label: found unsafe '$unexpected' in '$value'"
}

reset_test() {
    CALLS=()
    LOGS=()
    RUN_CMD_FAILURE=""
    unset DEVPANEL_INSTALL_COMPOSER DEVPANEL_COMPOSER_VERSION
    unset DEVPANEL_INSTALL_NODE_NVM DEVPANEL_NODE_VERSION
    unset REAL_USER USER_HOME
    NVM_INSTALL_VERSION="v0.40.1"
}

. "$ROOT_DIR/installation/dependencies/install_composer.sh"
. "$ROOT_DIR/installation/dependencies/install_node.sh"

reset_test
install_composer_if_requested
assert_eq "0" "${#CALLS[@]}" "Composer skip command count"
assert_contains "${LOGS[*]}" "Composer install skipped" "Composer skip log"

reset_test
DEVPANEL_INSTALL_COMPOSER=1
install_composer_if_requested
assert_eq "2" "${#CALLS[@]}" "Composer latest command count"
assert_contains "${CALLS[0]}" "apt-get install -y curl ca-certificates php-cli" "Composer prerequisites"
assert_contains "${CALLS[1]}" "composer.github.io/installer.sig" "Composer checksum source"
assert_contains "${CALLS[1]}" "hash_file" "Composer checksum calculation"
assert_contains "${CALLS[1]}" "sha384" "Composer checksum algorithm"
assert_contains "${CALLS[1]}" "--install-dir=/usr/local/bin" "Composer install directory"

reset_test
DEVPANEL_INSTALL_COMPOSER=1
DEVPANEL_COMPOSER_VERSION=2
install_composer_if_requested
assert_eq "3" "${#CALLS[@]}" "Composer 2 command count"
assert_contains "${CALLS[2]}" "composer self-update --2" "Composer 2 selection"

reset_test
DEVPANEL_INSTALL_COMPOSER=1
RUN_CMD_FAILURE="install Composer prerequisites"
if install_composer_if_requested; then
    fail "Composer install should stop when prerequisites fail"
fi
assert_eq "1" "${#CALLS[@]}" "Composer prerequisite failure command count"

reset_test
DEVPANEL_INSTALL_NODE_NVM=1
if install_node_nvm_if_requested; then
    fail "Node install should fail without target-user context"
fi
assert_eq "0" "${#CALLS[@]}" "Node missing-context command count"
assert_contains "${LOGS[*]}" "Cannot install NVM without REAL_USER and USER_HOME" "Node context error"

reset_test
DEVPANEL_INSTALL_NODE_NVM=1
REAL_USER=devpanel-user
USER_HOME=/home/devpanel-user
install_node_nvm_if_requested
assert_eq "2" "${#CALLS[@]}" "Node default command count"
assert_contains "${CALLS[0]}" "apt-get install -y curl ca-certificates" "Node prerequisites"
assert_contains "${CALLS[1]}" "sudo -u devpanel-user env HOME=/home/devpanel-user bash -lc" "Node target user"
assert_contains "${CALLS[1]}" "set -e -o pipefail" "Node pipeline failure handling"
assert_contains "${CALLS[1]}" "nvm install 22" "Node default install"
assert_contains "${CALLS[1]}" "nvm alias default 22" "Node default alias"
assert_contains "${CALLS[1]}" "nvm use 22" "Node active version"

reset_test
REAL_USER=devpanel-user
USER_HOME=/home/devpanel-user
unsafe_version="22'; touch /tmp/devpanel-injected; '"
install_node_nvm_version "$unsafe_version"
assert_contains "${CALLS[1]}" "22\\'\\;\\ touch\\ /tmp/devpanel-injected\\;\\ \\'" "Node version quoting"
assert_not_contains "${CALLS[1]}" "nvm install '22'; touch" "Node command boundary"

reset_test
DEVPANEL_INSTALL_NODE_NVM=1
REAL_USER=devpanel-user
USER_HOME=/home/devpanel-user
RUN_CMD_FAILURE="install NVM prerequisites"
if install_node_nvm_if_requested; then
    fail "Node install should stop when prerequisites fail"
fi
assert_eq "1" "${#CALLS[@]}" "Node prerequisite failure command count"

echo "Runtime installer shell tests passed"
