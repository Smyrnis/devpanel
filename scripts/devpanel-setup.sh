#!/bin/bash
# devpanel-setup.sh — First-run setup for DevPanel
# ─────────────────────────────────────────────────────────────────────────────
# What this does:
#   1. Creates ~/projects/                  → PHP project source code lives here
#   2. Copies index.php → /var/www/html/    → replaces Apache's default welcome
#                                             page; index.html is left untouched
#   3. Sets DirectoryIndex in 000-default.conf → php loads before html globally
#   4. Creates /etc/apache2/sites-available/devpanel.conf (empty, for vhosts)
#   5. Enables devpanel.conf via a2ensite
#   6. Enables mod_rewrite (required by most PHP frameworks)
#   7. Enables mod_phpX.Y for EVERY installed PHP version found on the system
#      so that per-VirtualHost SetHandler application/x-httpd-phpX.Y works
#      immediately without manual intervention.
#   8. Writes ~/.config/devpanel/config.toml
# ─────────────────────────────────────────────────────────────────────────────
set -e

TEAL='\033[0;36m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
RED='\033[0;31m'; BOLD='\033[1m'; NC='\033[0m'

log_info()  { echo -e "${TEAL}  →${NC} $1"; }
log_ok()    { echo -e "${GREEN}  ✓${NC} $1"; }
log_warn()  { echo -e "${YELLOW}  !${NC} $1"; }
log_step()  { echo -e "\n${BOLD}${TEAL}[$1]${NC} $2"; }
log_err()   { echo -e "${RED}  ✗${NC} $1"; }

if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Run with sudo: sudo bash devpanel-setup.sh${NC}"
    exit 1
fi
if [ -z "$SUDO_USER" ]; then
    echo -e "${RED}Error: Cannot detect target user. Use sudo, not direct root login.${NC}"
    exit 1
fi

REAL_USER="$SUDO_USER"
USER_HOME=$(eval echo "~$SUDO_USER")

PROJECTS_DIR="$USER_HOME/projects"
WEBROOT="/var/www/html"
APACHE2_CONF="/etc/apache2/apache2.conf"
DEVPANEL_CONF="/etc/apache2/sites-available/devpanel.conf"
CFG_DIR="$USER_HOME/.config/devpanel"
CFG_FILE="$CFG_DIR/config.toml"
DEFAULT_SITE="/etc/apache2/sites-available/000-default.conf"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INDEX_PHP_SRC="$SCRIPT_DIR/../share/index.php"
[ ! -f "$INDEX_PHP_SRC" ] && INDEX_PHP_SRC="/usr/share/devpanel/index.php"

echo ""
echo -e "${BOLD}${TEAL}╔══════════════════════════════════════╗${NC}"
echo -e "${BOLD}${TEAL}║       DevPanel Setup                 ║${NC}"
echo -e "${BOLD}${TEAL}╚══════════════════════════════════════╝${NC}"
echo ""
echo -e "  User:     ${BOLD}$REAL_USER${NC}"
echo -e "  Projects: ${BOLD}$PROJECTS_DIR${NC}"
echo -e "  Web root: ${BOLD}$WEBROOT${NC}"
echo -e "  VHosts:   ${BOLD}$DEVPANEL_CONF${NC}"
echo ""

if ! command -v apache2 &>/dev/null && ! command -v apachectl &>/dev/null; then
    log_err "Apache2 is not installed. Install it first:"
    echo -e "       ${BOLD}sudo apt-get install -y apache2 libapache2-mod-php${NC}"
    exit 1
fi

log_step "1/8" "Creating ~/projects/ directory"
if [ ! -d "$PROJECTS_DIR" ]; then
    mkdir -p "$PROJECTS_DIR"
    chown "$REAL_USER:www-data" "$PROJECTS_DIR"
    chmod 755 "$PROJECTS_DIR"
    log_ok "Created: $PROJECTS_DIR"
else
    chown "$REAL_USER:www-data" "$PROJECTS_DIR" 2>/dev/null || true
    log_info "Already exists: $PROJECTS_DIR"
fi

log_step "2/8" "Installing DevPanel welcome page → /var/www/html/index.php"
if [ -f "$INDEX_PHP_SRC" ]; then
    cp "$INDEX_PHP_SRC" "$WEBROOT/index.php"
    chown root:www-data "$WEBROOT/index.php"
    chmod 644 "$WEBROOT/index.php"
    log_ok "Copied index.php → $WEBROOT/index.php"
    log_info "index.html left intact at $WEBROOT/index.html"
else
    log_warn "index.php source not found at $INDEX_PHP_SRC — skipping"
    log_warn "Copy it manually: sudo cp share/index.php $WEBROOT/index.php"
fi

log_step "3/8" "Setting DirectoryIndex in 000-default.conf (index.php first)"
if [ -f "$DEFAULT_SITE" ]; then
    if [ ! -f "${DEFAULT_SITE}.devpanel.bak" ]; then
        cp "$DEFAULT_SITE" "${DEFAULT_SITE}.devpanel.bak"
        log_ok "Backup saved: ${DEFAULT_SITE}.devpanel.bak"
    fi
    if grep -q "DirectoryIndex" "$DEFAULT_SITE"; then
        sed -i 's|^\s*DirectoryIndex .*|    DirectoryIndex index.php index.html index.htm|' "$DEFAULT_SITE"
        log_ok "Updated existing DirectoryIndex → index.php first"
    else
        sed -i '/<VirtualHost \*:80>/a\    DirectoryIndex index.php index.html index.htm' "$DEFAULT_SITE"
        log_ok "Inserted DirectoryIndex into VirtualHost block"
    fi
else
    log_warn "000-default.conf not found at $DEFAULT_SITE — skipping"
fi

log_step "4/8" "Creating /etc/apache2/sites-available/devpanel.conf"
if [ ! -f "$DEVPANEL_CONF" ]; then
    cat > "$DEVPANEL_CONF" << 'APACHECONF'
# DevPanel managed VirtualHosts
# Managed by DevPanel — use the GUI to add/edit/remove entries below.
# Each <VirtualHost> block is one .local project. Add one in DevPanel → VirtualHosts.

APACHECONF
    log_ok "Created: $DEVPANEL_CONF"
else
    log_info "Already exists: $DEVPANEL_CONF (not overwritten)"
fi

log_step "5/8" "Enabling devpanel.conf"
if command -v a2ensite &>/dev/null; then
    a2ensite devpanel.conf 2>/dev/null \
        && log_ok "a2ensite devpanel.conf" \
        || log_info "devpanel.conf already enabled"
else
    ENABLED="/etc/apache2/sites-enabled/devpanel.conf"
    if [ ! -L "$ENABLED" ]; then
        ln -s "$DEVPANEL_CONF" "$ENABLED"
        log_ok "Created symlink: $ENABLED"
    else
        log_info "Symlink already exists"
    fi
fi

log_step "6/8" "Enabling mod_rewrite"
if command -v a2enmod &>/dev/null; then
    a2enmod rewrite 2>/dev/null && log_ok "mod_rewrite enabled" \
        || log_info "mod_rewrite already enabled"
else
    log_warn "a2enmod not found — enable mod_rewrite manually"
fi

log_step "7/8" "Enabling Apache mod_phpX.Y for all installed PHP versions"
PHP_MODS_ENABLED=0
PHP_MODS_FOUND=0

for ver in 7.4 8.0 8.1 8.2 8.3 8.4; do
    MOD_LOAD="/etc/apache2/mods-available/php${ver}.load"
    if [ -f "$MOD_LOAD" ]; then
        PHP_MODS_FOUND=$((PHP_MODS_FOUND + 1))
        if command -v a2enmod &>/dev/null; then
            if a2enmod "php${ver}" 2>/dev/null; then
                log_ok "Enabled: mod_php${ver}"
                PHP_MODS_ENABLED=$((PHP_MODS_ENABLED + 1))
            else
                log_info "mod_php${ver} already enabled or skipped"
                PHP_MODS_ENABLED=$((PHP_MODS_ENABLED + 1))
            fi
        else
            ENABLED_LINK="/etc/apache2/mods-enabled/php${ver}.load"
            CONF_LINK="/etc/apache2/mods-enabled/php${ver}.conf"
            if [ ! -L "$ENABLED_LINK" ]; then
                ln -s "$MOD_LOAD" "$ENABLED_LINK" 2>/dev/null && \
                    log_ok "Symlinked: $ENABLED_LINK" || true
            fi
            MOD_CONF="/etc/apache2/mods-available/php${ver}.conf"
            if [ -f "$MOD_CONF" ] && [ ! -L "$CONF_LINK" ]; then
                ln -s "$MOD_CONF" "$CONF_LINK" 2>/dev/null || true
            fi
            PHP_MODS_ENABLED=$((PHP_MODS_ENABLED + 1))
        fi
    fi
done

if [ "$PHP_MODS_FOUND" -eq 0 ]; then
    log_warn "No mod_phpX.Y found in /etc/apache2/mods-available/"
    log_warn "Install PHP with: sudo apt-get install -y libapache2-mod-php8.2"
    log_warn "Per-vhost PHP pinning (SetHandler) will not be available until then."
else
    log_ok "PHP Apache mods ready: $PHP_MODS_ENABLED/$PHP_MODS_FOUND version(s)"
fi

echo ""
log_info "Testing Apache configuration…"
if apache2ctl configtest 2>/dev/null; then
    log_ok "Apache configuration OK"
    if systemctl is-active --quiet apache2; then
        systemctl reload apache2 && log_ok "Apache reloaded" \
            || log_warn "Apache reload failed — try: sudo systemctl restart apache2"
    else
        systemctl start apache2 && log_ok "Apache started" \
            || log_warn "Apache start failed — check: sudo journalctl -u apache2 -n 20"
    fi
else
    log_warn "Apache config test failed. Fix errors before reloading."
    apache2ctl configtest 2>&1 | sed 's/^/    /'
fi

log_step "8/8" "Writing DevPanel configuration"
mkdir -p "$CFG_DIR"
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

if ! groups "$REAL_USER" | grep -q www-data; then
    usermod -aG www-data "$REAL_USER"
    log_ok "Added $REAL_USER to www-data group (re-login to take effect)"
fi

echo ""
echo -e "${TEAL}─────────────────────────────────────────────${NC}"
echo -e "${BOLD}${GREEN}Setup complete!${NC}"
echo ""
echo -e "  ${TEAL}→${NC} Open ${BOLD}http://localhost${NC} — DevPanel welcome page (index.php)"
echo -e "  ${TEAL}→${NC} index.html at $WEBROOT/index.html is unchanged"
echo -e "  ${TEAL}→${NC} Clone or create PHP projects in ${BOLD}$PROJECTS_DIR/${NC}"
echo -e "  ${TEAL}→${NC} Open DevPanel → ${BOLD}VirtualHosts${NC} to add project .local domains"
echo -e "  ${TEAL}→${NC} Use ${BOLD}Repos${NC} tab to clone from GitHub / Bitbucket"
if [ "$PHP_MODS_ENABLED" -gt 0 ]; then
echo -e "  ${TEAL}→${NC} Per-vhost PHP pinning is ${BOLD}ready${NC}: $PHP_MODS_ENABLED PHP mod(s) enabled"
echo -e "     Add ${BOLD}SetHandler application/x-httpd-phpX.Y${NC} inside a <Directory> block"
echo -e "     or use the ${BOLD}PHP Version${NC} dropdown in DevPanel → VirtualHosts"
fi
echo -e "${TEAL}─────────────────────────────────────────────${NC}"
echo ""