#!/bin/bash

set -e

LOG_DIR="/var/log/devpanel"
LOG_FILE="$LOG_DIR/setup.log"

mkdir -p "$LOG_DIR"
chmod 755 "$LOG_DIR"

: > "$LOG_FILE"
chmod 644 "$LOG_FILE"

_ts() { date '+%Y-%m-%d %H:%M:%S'; }

log_step()  { echo "$(_ts) [STEP]  $1" >> "$LOG_FILE"; }
log_info()  { echo "$(_ts) [INFO]  $1" >> "$LOG_FILE"; }
log_ok()    { echo "$(_ts) [OK]    $1" >> "$LOG_FILE"; }
log_warn()  { echo "$(_ts) [WARN]  $1" >> "$LOG_FILE"; }
log_err()   { echo "$(_ts) [ERROR] $1" >> "$LOG_FILE"; }
log_cmd()   { echo "$(_ts) [CMD]   $1" >> "$LOG_FILE"; }

run_cmd() {
    local description="$1"
    shift
    log_cmd "$description: $*"
    local output
    output=$("$@" 2>&1)
    local rc=$?
    if [ -n "$output" ]; then
        echo "$output" | while IFS= read -r line; do
            echo "$(_ts) [OUT]   $line" >> "$LOG_FILE"
        done
    fi
    return $rc
}

log_step "Starting DevPanel setup"

if [ "$EUID" -ne 0 ]; then
    log_err "Not running as root — aborting"
    exit 1
fi
if [ -z "$SUDO_USER" ]; then
    log_err "Cannot detect target user (SUDO_USER is empty) — aborting"
    exit 1
fi

REAL_USER="$SUDO_USER"
USER_HOME=$(eval echo "~$SUDO_USER")

PROJECTS_DIR="$USER_HOME/projects"
WEBROOT="/var/www/html"
DEVPANEL_CONF="/etc/apache2/sites-available/devpanel.conf"
DEFAULT_SITE="/etc/apache2/sites-available/000-default.conf"
CFG_DIR="$USER_HOME/.config/devpanel"
CFG_FILE="$CFG_DIR/config.toml"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INDEX_PHP_SRC="$SCRIPT_DIR/../share/index.php"
[ ! -f "$INDEX_PHP_SRC" ] && INDEX_PHP_SRC="/usr/share/devpanel/index.php"

log_info "User:     $REAL_USER"
log_info "Home:     $USER_HOME"
log_info "Projects: $PROJECTS_DIR"
log_info "Webroot:  $WEBROOT"
log_info "VHosts:   $DEVPANEL_CONF"

log_step "1/8 — Checking Apache installation"

if ! command -v apache2 &>/dev/null && ! command -v apachectl &>/dev/null; then
    log_err "Apache2 is not installed"
    log_err "Install it first: apt-get install -y apache2 libapache2-mod-php"
    exit 1
fi

log_ok "Apache2 found: $(command -v apache2 2>/dev/null || command -v apachectl)"

log_step "2/8 — Creating ~/projects/ directory"

if [ ! -d "$PROJECTS_DIR" ]; then
    if run_cmd "mkdir projects" mkdir -p "$PROJECTS_DIR"; then
        chown "$REAL_USER:www-data" "$PROJECTS_DIR" 2>>"$LOG_FILE" || true
        chmod 755 "$PROJECTS_DIR"
        log_ok "Created: $PROJECTS_DIR"
    else
        log_err "Failed to create: $PROJECTS_DIR"
    fi
else
    chown "$REAL_USER:www-data" "$PROJECTS_DIR" 2>>"$LOG_FILE" || true
    log_info "Already exists: $PROJECTS_DIR"
fi

log_step "3/8 — Installing DevPanel welcome page"

if [ -f "$INDEX_PHP_SRC" ]; then
    if run_cmd "copy index.php" cp "$INDEX_PHP_SRC" "$WEBROOT/index.php"; then
        chown root:www-data "$WEBROOT/index.php"
        chmod 644 "$WEBROOT/index.php"
        log_ok "Copied: $INDEX_PHP_SRC → $WEBROOT/index.php"
        log_info "index.html left intact at $WEBROOT/index.html"
    else
        log_warn "Failed to copy index.php — skipping"
    fi
else
    log_warn "index.php source not found at $INDEX_PHP_SRC"
    log_warn "Copy manually: cp share/index.php $WEBROOT/index.php"
fi

log_step "4/8 — Setting DirectoryIndex in 000-default.conf"

if [ -f "$DEFAULT_SITE" ]; then
    if [ ! -f "${DEFAULT_SITE}.devpanel.bak" ]; then
        run_cmd "backup 000-default.conf" cp "$DEFAULT_SITE" "${DEFAULT_SITE}.devpanel.bak" \
            && log_ok "Backup saved: ${DEFAULT_SITE}.devpanel.bak" \
            || log_warn "Backup failed — continuing anyway"
    else
        log_info "Backup already exists — skipping"
    fi

    if grep -q "DirectoryIndex" "$DEFAULT_SITE"; then
        run_cmd "update DirectoryIndex" \
            sed -i 's|^\s*DirectoryIndex .*|    DirectoryIndex index.php index.html index.htm|' "$DEFAULT_SITE" \
            && log_ok "Updated DirectoryIndex → index.php first" \
            || log_warn "sed failed on 000-default.conf"
    else
        run_cmd "insert DirectoryIndex" \
            sed -i '/<VirtualHost \*:80>/a\    DirectoryIndex index.php index.html index.htm' "$DEFAULT_SITE" \
            && log_ok "Inserted DirectoryIndex into VirtualHost block" \
            || log_warn "sed insert failed on 000-default.conf"
    fi
else
    log_warn "000-default.conf not found at $DEFAULT_SITE — skipping"
fi

log_step "5/8 — Creating devpanel.conf"

if [ ! -f "$DEVPANEL_CONF" ]; then
    cat > "$DEVPANEL_CONF" << 'APACHECONF'
# DevPanel managed VirtualHosts
# Managed by DevPanel — use the GUI to add/edit/remove entries below.
# Each <VirtualHost> block is one .local project.

APACHECONF
    log_ok "Created: $DEVPANEL_CONF"
else
    log_info "Already exists: $DEVPANEL_CONF (not overwritten)"
fi

log_step "6/8 — Enabling devpanel.conf"

if command -v a2ensite &>/dev/null; then
    if run_cmd "a2ensite devpanel.conf" a2ensite devpanel.conf; then
        log_ok "a2ensite devpanel.conf succeeded"
    else
        log_info "devpanel.conf may already be enabled"
    fi
else
    ENABLED_LINK="/etc/apache2/sites-enabled/devpanel.conf"
    if [ ! -L "$ENABLED_LINK" ]; then
        run_cmd "symlink devpanel.conf" ln -s "$DEVPANEL_CONF" "$ENABLED_LINK" \
            && log_ok "Created symlink: $ENABLED_LINK" \
            || log_err "Failed to create symlink: $ENABLED_LINK"
    else
        log_info "Symlink already exists: $ENABLED_LINK"
    fi
fi

log_step "7/8 — Enabling mod_rewrite"

if command -v a2enmod &>/dev/null; then
    if run_cmd "a2enmod rewrite" a2enmod rewrite; then
        log_ok "mod_rewrite enabled"
    else
        log_info "mod_rewrite already enabled or not available"
    fi
else
    log_warn "a2enmod not found — enable mod_rewrite manually"
fi

log_step "8/8 — Enabling Apache mod_phpX.Y for all installed PHP versions"

PHP_MODS_FOUND=0
PHP_MODS_ENABLED=0

for ver in 5.6 7.4 8.0 8.1 8.2 8.3 8.4; do
    if [ "$ver" = "5.6" ]; then
        if [ -f "/etc/apache2/mods-available/php5.6.load" ]; then
            MOD_NAME="php5.6"
        elif [ -f "/etc/apache2/mods-available/php5.load" ]; then
            MOD_NAME="php5"
        else
            log_info "PHP 5.6 — no Apache mod found in mods-available, skipping"
            continue
        fi
    else
        MOD_NAME="php${ver}"
    fi

    MOD_LOAD="/etc/apache2/mods-available/${MOD_NAME}.load"

    if [ ! -f "$MOD_LOAD" ]; then
        log_info "PHP $ver — $MOD_LOAD not found, skipping"
        continue
    fi

    PHP_MODS_FOUND=$((PHP_MODS_FOUND + 1))

    if command -v a2enmod &>/dev/null; then
        if run_cmd "a2enmod $MOD_NAME" a2enmod "$MOD_NAME"; then
            log_ok "Enabled mod_${MOD_NAME} (PHP $ver)"
            PHP_MODS_ENABLED=$((PHP_MODS_ENABLED + 1))
        else
            log_info "mod_${MOD_NAME} already enabled or skipped"
            PHP_MODS_ENABLED=$((PHP_MODS_ENABLED + 1))
        fi
    else
        ENABLED_LINK="/etc/apache2/mods-enabled/${MOD_NAME}.load"
        CONF_SRC="/etc/apache2/mods-available/${MOD_NAME}.conf"
        CONF_LINK="/etc/apache2/mods-enabled/${MOD_NAME}.conf"

        if [ ! -L "$ENABLED_LINK" ]; then
            run_cmd "symlink $MOD_NAME.load" ln -s "$MOD_LOAD" "$ENABLED_LINK" \
                && log_ok "Symlinked: $ENABLED_LINK" \
                || log_warn "Failed to symlink: $ENABLED_LINK"
        else
            log_info "Already symlinked: $ENABLED_LINK"
        fi

        if [ -f "$CONF_SRC" ] && [ ! -L "$CONF_LINK" ]; then
            run_cmd "symlink $MOD_NAME.conf" ln -s "$CONF_SRC" "$CONF_LINK" || true
        fi

        PHP_MODS_ENABLED=$((PHP_MODS_ENABLED + 1))
    fi
done

if [ "$PHP_MODS_FOUND" -eq 0 ]; then
    log_warn "No mod_phpX.Y found — per-vhost PHP pinning will not be available"
    log_warn "Install PHP: apt-get install -y libapache2-mod-php8.2"
else
    log_ok "PHP Apache mods: $PHP_MODS_ENABLED/$PHP_MODS_FOUND version(s) enabled"
fi

log_info "Testing Apache configuration"

if run_cmd "apache2ctl configtest" apache2ctl configtest; then
    log_ok "Apache configuration OK"
    if systemctl is-active --quiet apache2; then
        if run_cmd "systemctl reload apache2" systemctl reload apache2; then
            log_ok "Apache reloaded"
        else
            log_warn "Apache reload failed — try: systemctl restart apache2"
        fi
    else
        if run_cmd "systemctl start apache2" systemctl start apache2; then
            log_ok "Apache started"
        else
            log_warn "Apache start failed — check: journalctl -u apache2 -n 20"
        fi
    fi
else
    log_warn "Apache config test failed — fix errors before reloading"
fi

log_info "Writing DevPanel configuration"

if run_cmd "mkdir config dir" mkdir -p "$CFG_DIR"; then
    chown -R "$REAL_USER" "$CFG_DIR"
    cat > "$CFG_FILE" << TOML
# DevPanel configuration
# Generated by devpanel-setup.sh on $(date '+%Y-%m-%d %H:%M:%S')

repos_root    = "$PROJECTS_DIR"
devpanel_conf = "$DEVPANEL_CONF"
hosts_file    = "/etc/hosts"
TOML
    chown "$REAL_USER" "$CFG_FILE"
    log_ok "Config written: $CFG_FILE"
else
    log_err "Failed to create config directory: $CFG_DIR"
fi

if ! groups "$REAL_USER" | grep -q www-data; then
    if run_cmd "usermod www-data" usermod -aG www-data "$REAL_USER"; then
        log_ok "Added $REAL_USER to www-data group (re-login to take effect)"
    else
        log_warn "Failed to add $REAL_USER to www-data group"
    fi
else
    log_info "$REAL_USER is already in www-data group"
fi

chown "$REAL_USER" "$LOG_FILE" 2>/dev/null || true

log_ok "Setup complete"
log_info "Log file: $LOG_FILE"

exit 0
