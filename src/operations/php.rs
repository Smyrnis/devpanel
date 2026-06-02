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
            format!("libapache2-mod-php{version}"),
            format!("php{version}-mysql"),
            format!("php{version}-xml"),
            format!("php{version}-mbstring"),
        ]
    } else {
        vec![
            format!("php{version}"),
            format!("libapache2-mod-php{version}"),
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
    super::run(password, &["sh", "-c", &full_cmd]).await
}

pub async fn switch_php(password: &str, version: &str) -> Result<String, String> {
    let apache_module = apache_module_for_version(version);
    if apache_module.is_none() && !crate::core::dry_run::active() {
        return Err(format!(
            "Apache PHP module for PHP {version} was not found. Install libapache2-mod-php{version} first."
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

    if let Some(selected) = apache_module {
        for module in known_php_apache_modules() {
            if module != selected && apache_module_available(&module) {
                let _ = super::run(password, &["a2dismod", &module]).await;
            }
        }
        super::run(password, &["a2enmod", &selected]).await?;
        let _ = super::systemctl(password, "reload", "apache2").await;
    }

    Ok(format!("PHP {version} selected for CLI and Apache"))
}

fn apache_module_for_version(version: &str) -> Option<String> {
    let spec = crate::core::app_config::php_versions()
        .into_iter()
        .find(|spec| spec.version == version)?;

    if crate::core::dry_run::active() {
        return (!spec.apache_module.is_empty()).then_some(spec.apache_module);
    }

    if apache_module_available(&spec.apache_module) {
        return Some(spec.apache_module);
    }

    spec.legacy_apache_module
        .filter(|module| apache_module_available(module))
}

fn known_php_apache_modules() -> Vec<String> {
    let mut modules = Vec::new();
    for spec in crate::core::app_config::php_versions() {
        if !spec.apache_module.is_empty() && !modules.contains(&spec.apache_module) {
            modules.push(spec.apache_module);
        }
        if let Some(legacy) = spec.legacy_apache_module
            && !modules.contains(&legacy)
        {
            modules.push(legacy);
        }
    }
    modules
}

fn apache_module_available(module: &str) -> bool {
    crate::core::dry_run::active()
        || std::path::Path::new(&paths::apache_mod_available(module)).exists()
}
