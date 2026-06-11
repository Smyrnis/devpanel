#!/bin/bash

set -euo pipefail

package="${1:-}"
if [ -z "$package" ]; then
    package="$(find target/debian -maxdepth 1 -type f -name 'devpanel_*.deb' -print -quit 2>/dev/null || true)"
fi

if [ -z "$package" ] || [ ! -f "$package" ]; then
    echo "DevPanel Debian package not found" >&2
    exit 1
fi

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

dpkg-deb --extract "$package" "$work_dir/root"
dpkg-deb --control "$package" "$work_dir/control"

require_file() {
    local path="$1"
    if [ ! -f "$work_dir/root/$path" ]; then
        echo "Missing package file: /$path" >&2
        exit 1
    fi
}

require_mode() {
    local path="$1"
    local expected="$2"
    local actual
    actual="$(stat -c '%a' "$work_dir/root/$path")"
    if [ "$actual" != "$expected" ]; then
        echo "Invalid mode for /$path: expected $expected, got $actual" >&2
        exit 1
    fi
}

require_file "usr/bin/devpanel"
require_file "usr/share/applications/devpanel.desktop"
require_file "usr/share/pixmaps/devpanel.png"
require_file "usr/share/devpanel/versions/composer.json"
require_file "usr/share/devpanel/versions/node.json"
require_file "usr/share/devpanel/installation/dependencies/install_composer.sh"
require_file "usr/share/devpanel/installation/dependencies/install_node.sh"

require_mode "usr/bin/devpanel" "755"
require_mode "usr/share/devpanel/installation/dependencies/install_composer.sh" "755"
require_mode "usr/share/devpanel/installation/dependencies/install_node.sh" "755"
require_mode "usr/share/devpanel/versions/composer.json" "644"
require_mode "usr/share/devpanel/versions/node.json" "644"

grep -q '^Package: devpanel$' "$work_dir/control/control"
grep -Eq '^Depends: .*acl' "$work_dir/control/control"
grep -q '^Icon=devpanel$' "$work_dir/root/usr/share/applications/devpanel.desktop"
test -x "$work_dir/control/postinst"
bash -n "$work_dir/control/postinst"

echo "Verified Debian package: $package"
