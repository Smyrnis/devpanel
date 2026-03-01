#!/bin/bash
# devpanel-setup.sh — First-run setup for DevPanel
# ─────────────────────────────────────────────────────────────────────────────
# What this does:
#   1. Creates ~/projects/                  → PHP project source code lives here
#   2. Copies index.php → /var/www/html/    → replaces Apache's default welcome
#                                             page; index.html is left untouched
#   3. Sets DirectoryIndex in apache2.conf  → php loads before html globally
#   4. Creates /etc/apache2/sites-available/devpanel.conf (empty, for vhosts)
#   5. Enables devpanel.conf via a2ensite
#   6. Enables mod_rewrite (required by most PHP frameworks)
#   7. Writes ~/.config/devpanel/config.toml
# ─────────────────────────────────────────────────────────────────────────────
set -e

TEAL='\033[0;36m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
RED='\033[0;31m'; BOLD='\033[1m'; NC='\033[0m'

log_info()  { echo -e "${TEAL}  →${NC} $1"; }
log_ok()    { echo -e "${GREEN}  ✓${NC} $1"; }
log_warn()  { echo -e "${YELLOW}  !${NC} $1"; }
log_step()  { echo -e "\n${BOLD}${TEAL}[$1]${NC} $2"; }
log_err()   { echo -e "${RED}  ✗${NC} $1"; }

# ── Root / user check ──────────────────────────────────────────────────────────
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

# ── Paths ──────────────────────────────────────────────────────────────────────
PROJECTS_DIR="$USER_HOME/projects"
WEBROOT="/var/www/html"
APACHE2_CONF="/etc/apache2/apache2.conf"
DEVPANEL_CONF="/etc/apache2/sites-available/devpanel.conf"
CFG_DIR="$USER_HOME/.config/devpanel"
CFG_FILE="$CFG_DIR/config.toml"
DEFAULT_SITE="/etc/apache2/sites-available/000-default.conf"

# Locate index.php relative to this script, then fall back to deb install path
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

# ── Check Apache is installed ──────────────────────────────────────────────────
if ! command -v apache2 &>/dev/null && ! command -v apachectl &>/dev/null; then
    log_err "Apache2 is not installed. Install it first:"
    echo -e "       ${BOLD}sudo apt-get install -y apache2 libapache2-mod-php${NC}"
    exit 1
fi

# ── Step 1: Create ~/projects/ ────────────────────────────────────────────────
log_step "1/7" "Creating ~/projects/ directory"
if [ ! -d "$PROJECTS_DIR" ]; then
    mkdir -p "$PROJECTS_DIR"
    chown "$REAL_USER:www-data" "$PROJECTS_DIR"
    chmod 755 "$PROJECTS_DIR"
    log_ok "Created: $PROJECTS_DIR"
else
    chown "$REAL_USER:www-data" "$PROJECTS_DIR" 2>/dev/null || true
    log_info "Already exists: $PROJECTS_DIR"
fi

# ── Step 2: Deploy index.php into /var/www/html ────────────────────────────────
# index.html is NOT removed — it stays as a fallback.
# We add index.php alongside it and ensure apache2.conf serves php first.
log_step "2/7" "Installing DevPanel welcome page → /var/www/html/index.php"
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

# ── Step 3: Set DirectoryIndex in 000-default.conf ──────────────────────────

log_step "3/7" "Setting DirectoryIndex in 000-default.conf (index.php first)"

if [ -f "$DEFAULT_SITE" ]; then

    # Backup once
    if [ ! -f "${DEFAULT_SITE}.devpanel.bak" ]; then
        cp "$DEFAULT_SITE" "${DEFAULT_SITE}.devpanel.bak"
        log_ok "Backup saved: ${DEFAULT_SITE}.devpanel.bak"
    fi

    # If DirectoryIndex already exists inside VirtualHost, replace it
    if grep -q "DirectoryIndex" "$DEFAULT_SITE"; then
        sed -i 's|^\s*DirectoryIndex .*|    DirectoryIndex index.php index.html index.htm|' "$DEFAULT_SITE"
        log_ok "Updated existing DirectoryIndex → index.php first"
    else
        # Insert inside <VirtualHost *:80> block
        sed -i '/<VirtualHost \*:80>/a\    DirectoryIndex index.php index.html index.htm' "$DEFAULT_SITE"
        log_ok "Inserted DirectoryIndex into VirtualHost block"
    fi

else
    log_warn "000-default.conf not found at $DEFAULT_SITE — skipping"
fi

# ── Step 4: Create devpanel.conf ──────────────────────────────────────────────
log_step "4/7" "Creating /etc/apache2/sites-available/devpanel.conf"
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

# ── Step 5: Enable devpanel.conf ──────────────────────────────────────────────
log_step "5/7" "Enabling devpanel.conf"
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

# ── Step 6: Enable mod_rewrite ────────────────────────────────────────────────
log_step "6/7" "Enabling mod_rewrite and mod_php"
if command -v a2enmod &>/dev/null; then
    a2enmod rewrite 2>/dev/null && log_ok "mod_rewrite enabled" \
        || log_info "mod_rewrite already enabled"
    # Enable PHP module if available (libapache2-mod-php installs phpX.Y)
    for phpmod in php8.4 php8.3 php8.2 php8.1 php8.0 php7.4; do
        if a2enmod "$phpmod" 2>/dev/null; then
            log_ok "Apache PHP module enabled: $phpmod"
            break
        fi
    done
else
    log_warn "a2enmod not found — enable mod_rewrite manually"
fi

# ── Config test + reload ───────────────────────────────────────────────────────
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

# ── Step 7: Write DevPanel config.toml ────────────────────────────────────────
log_step "7/7" "Writing DevPanel configuration"
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

# ── Add user to www-data group ─────────────────────────────────────────────────
if ! groups "$REAL_USER" | grep -q www-data; then
    usermod -aG www-data "$REAL_USER"
    log_ok "Added $REAL_USER to www-data group (re-login to take effect)"
fi

# ── Summary ────────────────────────────────────────────────────────────────────
echo ""
echo -e "${TEAL}─────────────────────────────────────────────${NC}"
echo -e "${BOLD}${GREEN}Setup complete!${NC}"
echo ""
echo -e "  ${TEAL}→${NC} Open ${BOLD}http://localhost${NC} — DevPanel welcome page (index.php)"
echo -e "  ${TEAL}→${NC} index.html at $WEBROOT/index.html is unchanged"
echo -e "  ${TEAL}→${NC} Clone or create PHP projects in ${BOLD}$PROJECTS_DIR/${NC}"
echo -e "  ${TEAL}→${NC} Open DevPanel → ${BOLD}VirtualHosts${NC} to add project .local domains"
echo -e "  ${TEAL}→${NC} Use ${BOLD}Repos${NC} tab to clone from GitHub / Bitbucket"
echo -e "${TEAL}─────────────────────────────────────────────${NC}"
echo ""
