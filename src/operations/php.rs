use crate::core::paths;

pub async fn apt_php_op(password: &str, version: &str, install: bool) -> Result<String, String> {
    super::run(password, &["apt-get", "update"]).await?;

    let op = if install { "install" } else { "remove" };
    let pkg = if version == "5.6" {
        if install {
            "php5.6 php5.6-cli php5.6-common php5.6-mysql php5.6-xml php5.6-mbstring php5.6-curl"
                .to_string()
        } else {
            "php5.6 php5.6-*".to_string()
        }
    } else if install {
        format!(
            "php{0} php{0}-cli php{0}-common php{0}-mysql php{0}-xml php{0}-mbstring",
            version
        )
    } else {
        format!("php{0} php{0}-*", version)
    };

    let full_cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get -y {} {}", op, pkg);
    super::run(password, &["sh", "-c", &full_cmd]).await
}

pub async fn switch_php(password: &str, version: &str) -> Result<String, String> {
    let bin = if version == "5.6" {
        if std::path::Path::new(&paths::php_binary("5.6")).exists() {
            paths::php_binary("5.6")
        } else {
            paths::php_binary("5")
        }
    } else {
        paths::php_binary(version)
    };

    super::run(password, &["update-alternatives", "--set", "php", &bin]).await
}
