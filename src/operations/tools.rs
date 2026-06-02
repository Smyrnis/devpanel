use crate::core::paths;

pub async fn apt_update(password: &str) -> Result<String, String> {
    super::run(password, &["apt-get", "update"]).await
}

pub async fn apt_install_packages(password: &str, packages: &[&str]) -> Result<String, String> {
    let pkg_list = packages.join(" ");
    let install_cmd = format!(
        "DEBIAN_FRONTEND=noninteractive apt-get install -y {}",
        pkg_list
    );
    super::run(password, &["sh", "-c", &install_cmd]).await
}

pub async fn apt_package_op(
    password: &str,
    package: &str,
    install: bool,
) -> Result<String, String> {
    let op = if install { "install" } else { "remove" };
    super::run(password, &["apt-get", op, "-y", package]).await
}

pub async fn composer_op(password: &str, update: bool) -> Result<String, String> {
    if update {
        return super::run(password, &["composer", "self-update"]).await;
    }

    let cmd = format!(
        "php -r \"copy('https://getcomposer.org/installer', '/tmp/composer-setup.php');\" \
         && php /tmp/composer-setup.php --install-dir={} --filename=composer \
         && rm -f /tmp/composer-setup.php",
        paths::COMPOSER_INSTALL_DIR,
    );
    super::run(password, &["sh", "-c", &cmd]).await
}

pub async fn redis_service_op(
    password: &str,
    action: &str,
    service: &str,
) -> Result<String, String> {
    super::systemctl(password, action, service).await
}
