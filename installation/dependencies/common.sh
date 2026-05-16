#!/bin/bash

# Compatibility facade for older callers that source dependencies/common.sh.
# New code should source installation/lib/*.sh directly.

set -e

COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALLATION_DIR="$(cd "$COMMON_DIR/.." && pwd)"

# shellcheck source=/dev/null
. "$INSTALLATION_DIR/lib/paths.sh"

LOG_DIR="${LOG_DIR:-$DEV_LOG_DIR}"
LOG_FILE="${LOG_FILE:-$DEV_LOG_FILE}"

# shellcheck source=/dev/null
. "$INSTALLATION_DIR/lib/log.sh"
# shellcheck source=/dev/null
. "$INSTALLATION_DIR/lib/runner.sh"
# shellcheck source=/dev/null
. "$INSTALLATION_DIR/lib/context.sh"

init_log() {
    log_init
}
