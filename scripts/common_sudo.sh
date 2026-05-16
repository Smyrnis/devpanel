#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
. "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=/dev/null
. "$SCRIPT_DIR/lib/paths.sh"
