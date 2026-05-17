use super::common_sudo;

pub async fn reload(password: &str) -> Result<String, String> {
    common_sudo::systemctl(password, "reload", "apache2").await
}

pub async fn start(password: &str) -> Result<String, String> {
    common_sudo::systemctl(password, "start", "apache2").await
}

pub async fn enable_module(password: &str, name: &str) -> Result<String, String> {
    common_sudo::run(password, &["a2enmod", name]).await
}

pub async fn set_module(password: &str, name: &str, enable: bool) -> Result<String, String> {
    let cmd = if enable { "a2enmod" } else { "a2dismod" };
    common_sudo::run(password, &[cmd, name]).await
}

pub async fn set_module_and_reload(
    password: &str,
    name: &str,
    enable: bool,
) -> Result<String, String> {
    set_module(password, name, enable).await?;
    let _ = reload(password).await;
    Ok(String::new())
}
