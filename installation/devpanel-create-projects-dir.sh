#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR/lib"
DEPS_DIR="$SCRIPT_DIR/dependencies"

# shellcheck source=/dev/null
. "$LIB_DIR/paths.sh"

LOG_DIR="${LOG_DIR:-$DEV_LOG_DIR}"
LOG_FILE="${LOG_FILE:-$DEV_LOG_FILE}"

# shellcheck source=/dev/null
. "$LIB_DIR/log.sh"
# shellcheck source=/dev/null
. "$LIB_DIR/runner.sh"
# shellcheck source=/dev/null
. "$LIB_DIR/context.sh"
# shellcheck source=/dev/null
. "$DEPS_DIR/install_tools.sh"

log_init
log_step "Starting DevPanel projects directory setup"
require_root
detect_target_user
log_context

create_projects_dir
write_devpanel_config

chown "$REAL_USER" "$LOG_FILE" 2>/dev/null || true
log_ok "Projects directory setup complete"
log_info "Log file: $LOG_FILE"

exit 0
