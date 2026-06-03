use crate::core::paths;

pub async fn ensure_ondrej_php_ppa(password: &str) -> Result<String, String> {
    if crate::core::dry_run::active() {
        return Ok("Would ensure ppa:ondrej/php is configured".to_string());
    }

    let check = "grep -Rhs 'ondrej/php' /etc/apt/sources.list /etc/apt/sources.list.d/*.list /etc/apt/sources.list.d/*.sources >/dev/null 2>&1";
    if super::run(password, &["sh", "-c", check]).await.is_ok() {
        return Ok("ppa:ondrej/php already configured".to_string());
    }

    super::run(
        password,
        &[
            "sh",
            "-c",
            "DEBIAN_FRONTEND=noninteractive apt-get install -y software-properties-common ca-certificates",
        ],
    )
    .await?;
    super::run(password, &["add-apt-repository", "-y", "ppa:ondrej/php"]).await?;
    super::run(password, &["apt-get", "update"]).await?;

    Ok("ppa:ondrej/php configured".to_string())
}

pub fn php_packages_for_version(version: &str, install: bool) -> Vec<String> {
    if install {
        vec![
            format!("php{version}"),
            format!("php{version}-cli"),
            format!("php{version}-common"),
            php_fpm_package(version),
            format!("php{version}-mysql"),
            format!("php{version}-xml"),
            format!("php{version}-mbstring"),
        ]
    } else {
        vec![
            format!("php{version}"),
            php_fpm_package(version),
            format!("php{version}-*"),
        ]
    }
}

pub async fn apt_php_op(password: &str, version: &str, install: bool) -> Result<String, String> {
    if install {
        ensure_ondrej_php_ppa(password).await?;
    }

    super::run(password, &["apt-get", "update"]).await?;

    let op = if install { "install" } else { "remove" };
    let pkg = php_packages_for_version(version, install).join(" ");

    let full_cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get -y {} {}", op, pkg);
    let output = super::run(password, &["sh", "-c", &full_cmd]).await?;

    if install {
        configure_php_fpm_for_apache(password, version).await?;
        let _ = super::systemctl(password, "reload", "apache2").await;
    }

    Ok(output)
}

pub async fn switch_php(password: &str, version: &str) -> Result<String, String> {
    let fpm_conf = php_fpm_conf(version);
    if !php_fpm_conf_available(version) && !crate::core::dry_run::active() {
        return Err(format!(
            "Apache PHP-FPM config for PHP {version} was not found. Install php{version}-fpm first."
        ));
    }

    let bin = if version == "5.6" {
        if std::path::Path::new(&paths::php_binary("5.6")).exists() {
            paths::php_binary("5.6")
        } else {
            paths::php_binary("5")
        }
    } else {
        paths::php_binary(version)
    };

    super::run(password, &["update-alternatives", "--set", "php", &bin]).await?;

    configure_php_fpm_for_apache(password, version).await?;

    for conf in known_php_fpm_confs() {
        if conf != fpm_conf && php_fpm_conf_available_for_name(&conf) {
            let _ = super::run(password, &["a2disconf", &conf]).await;
        }
    }

    let _ = super::systemctl(password, "reload", "apache2").await;

    Ok(format!("PHP {version} selected for CLI and PHP-FPM"))
}

async fn configure_php_fpm_for_apache(password: &str, version: &str) -> Result<(), String> {
    super::run(password, &["a2enmod", "proxy_fcgi", "setenvif"]).await?;
    super::run(password, &["a2enconf", &php_fpm_conf(version)]).await?;
    let _ = super::run(
        password,
        &["systemctl", "enable", "--now", &php_fpm_service(version)],
    )
    .await;
    Ok(())
}

pub fn php_fpm_package(version: &str) -> String {
    php_spec(version)
        .map(|spec| spec.fpm_package)
        .unwrap_or_else(|| format!("php{version}-fpm"))
}

pub fn php_fpm_service(version: &str) -> String {
    php_spec(version)
        .map(|spec| spec.fpm_service)
        .unwrap_or_else(|| format!("php{version}-fpm"))
}

pub fn php_fpm_conf(version: &str) -> String {
    php_spec(version)
        .map(|spec| spec.fpm_conf)
        .unwrap_or_else(|| format!("php{version}-fpm"))
}

pub fn php_fpm_socket(version: &str) -> String {
    php_spec(version)
        .map(|spec| spec.fpm_socket)
        .unwrap_or_else(|| format!("/run/php/php{version}-fpm.sock"))
}

pub fn php_fpm_set_handler(version: &str) -> String {
    format!(
        "\"proxy:unix:{}|fcgi://localhost/\"",
        php_fpm_socket(version)
    )
}

pub fn php_fpm_conf_available(version: &str) -> bool {
    crate::core::dry_run::active() || php_fpm_conf_available_for_name(&php_fpm_conf(version))
}

fn known_php_fpm_confs() -> Vec<String> {
    let mut confs = Vec::new();
    for spec in crate::core::app_config::php_versions() {
        let conf = php_fpm_conf(&spec.version);
        if !confs.contains(&conf) {
            confs.push(conf);
        }
    }
    confs
}

fn php_fpm_conf_available_for_name(conf: &str) -> bool {
    std::path::Path::new(&paths::apache_conf_available(conf)).exists()
}

fn php_spec(version: &str) -> Option<crate::core::app_config::PhpVersionSpec> {
    crate::core::app_config::php_versions()
        .into_iter()
        .find(|spec| spec.version == version)
}
