#!/bin/bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

declare -a CALLS=()
declare -a LOGS=()

log_step() { LOGS+=("STEP|$*"); }
log_info() { LOGS+=("INFO|$*"); }
log_ok() { LOGS+=("OK|$*"); }
log_err() { LOGS+=("ERROR|$*"); }

command_exists() {
    [ "$1" = "setfacl" ] && [ "${SETFACL_AVAILABLE:-1}" = "1" ]
}

run_cmd() {
    local description="$1"
    shift
    CALLS+=("$description|$*")
}

safe_chown() {
    CALLS+=("chown|$*")
}

safe_chmod() {
    CALLS+=("chmod|$*")
}

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_contains() {
    local value="$1"
    local expected="$2"
    [[ "$value" == *"$expected"* ]] || fail "Missing '$expected' in '$value'"
}

. "$ROOT_DIR/installation/dependencies/install_tools.sh"

REAL_USER=devpanel-user
USER_HOME=/home/devpanel-user
PROJECTS_DIR=/home/devpanel-user/projects

configure_projects_runtime_access

assert_contains "${CALLS[*]}" "setfacl -m u:www-data:--x /home/devpanel-user"
assert_contains "${CALLS[*]}" "setfacl -R -m u:www-data:rX /home/devpanel-user/projects"
assert_contains "${CALLS[*]}" "find /home/devpanel-user/projects -type d -exec setfacl -m d:u:www-data:rX"

CALLS=()
LOGS=()
SETFACL_AVAILABLE=0
if configure_projects_runtime_access; then
    fail "Permission setup should fail without setfacl"
fi
assert_contains "${LOGS[*]}" "setfacl is required"

echo "Project permission shell tests passed"
