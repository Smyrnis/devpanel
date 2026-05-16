#!/bin/bash

DEV_LOG_DIR="/var/log/devpanel"
DEV_LOG_FILE="$DEV_LOG_DIR/setup.log"
DEV_WEBROOT="/var/www/html"
DEV_HOSTS_FILE="/etc/hosts"
DEV_APACHE_DIR="/etc/apache2"
DEV_APACHE_SITES_AVAILABLE="$DEV_APACHE_DIR/sites-available"
DEV_APACHE_SITES_ENABLED="$DEV_APACHE_DIR/sites-enabled"
DEV_APACHE_MODS_AVAILABLE="$DEV_APACHE_DIR/mods-available"
DEV_APACHE_MODS_ENABLED="$DEV_APACHE_DIR/mods-enabled"
DEV_DEFAULT_SITE="$DEV_APACHE_SITES_AVAILABLE/000-default.conf"
DEV_VHOST_CONF="$DEV_APACHE_SITES_AVAILABLE/devpanel.conf"
DEV_SHARE_DIR="/usr/share/devpanel"

devpanel_config_dir_for_home() {
    printf '%s/.config/devpanel' "$1"
}

devpanel_config_file_for_home() {
    printf '%s/config.toml' "$(devpanel_config_dir_for_home "$1")"
}
