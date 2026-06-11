#!/bin/bash

install_composer_if_requested() {
    if [ "${DEVPANEL_INSTALL_COMPOSER:-0}" != "1" ]; then
        log_info "Composer install skipped"
        return 0
    fi

    local version="${DEVPANEL_COMPOSER_VERSION:-latest}"
    install_composer_version "$version"
}

install_composer_version() {
    local version="${1:-latest}"
    log_step "Installing Composer $version"

    run_cmd "install Composer prerequisites" apt-get install -y curl ca-certificates php-cli \
        || return 1

    local cmd
    cmd="set -e"
    cmd="$cmd; trap 'rm -f /tmp/composer-setup.php' EXIT"
    cmd="$cmd; expected_checksum=\"\$(php -r 'copy(\"https://composer.github.io/installer.sig\", \"php://stdout\");')\""
    cmd="$cmd; php -r \"copy('https://getcomposer.org/installer', '/tmp/composer-setup.php');\""
    cmd="$cmd; actual_checksum=\"\$(php -r \"echo hash_file('sha384', '/tmp/composer-setup.php');\")\""
    cmd="$cmd; if [ \"\$expected_checksum\" != \"\$actual_checksum\" ]; then echo 'ERROR: Invalid Composer installer checksum' >&2; exit 1; fi"
    cmd="$cmd; php /tmp/composer-setup.php --install-dir=/usr/local/bin --filename=composer"

    run_cmd "install Composer" sh -c "$cmd" || return 1

    if [ "$version" = "1" ]; then
        run_cmd "select Composer 1" composer self-update --1 || return 1
    elif [ "$version" = "2" ]; then
        run_cmd "select Composer 2" composer self-update --2 || return 1
    elif [ "$version" != "latest" ]; then
        run_cmd "select Composer $version" composer self-update "$version" || return 1
    fi

    log_ok "Composer ready"
}
