#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEPS_DIR="$SCRIPT_DIR/dependencies"
LIB_DIR="$SCRIPT_DIR/lib"

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
. "$DEPS_DIR/install_apache.sh"
# shellcheck source=/dev/null
. "$DEPS_DIR/install_php.sh"
# shellcheck source=/dev/null
. "$DEPS_DIR/install_mysql.sh"
# shellcheck source=/dev/null
. "$DEPS_DIR/setup_vhost.sh"
# shellcheck source=/dev/null
. "$DEPS_DIR/install_tools.sh"

log_init
log_step "Starting DevPanel setup"
require_root
detect_target_user
resolve_index_php "$ROOT_DIR"
log_context

install_apache_check
create_projects_dir
install_welcome_page
configure_default_site
create_devpanel_conf
enable_devpanel_site
enable_rewrite
enable_php_modules
check_mysql
reload_or_start_apache
write_devpanel_config
ensure_www_data_group

chown "$REAL_USER" "$LOG_FILE" 2>/dev/null || true
log_ok "Setup complete"
log_info "Log file: $LOG_FILE"

exit 0
