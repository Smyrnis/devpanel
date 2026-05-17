pub async fn write_config(password: &str, path: &str, content: &str) -> Result<(), String> {
    super::write_file(password, path, content).await
}

pub async fn append_host(password: &str, hostname: &str) -> Result<(), String> {
    super::append_file(
        password,
        crate::core::paths::HOSTS_FILE,
        &format!("127.0.0.1    {}\n", hostname),
    )
    .await
}

pub async fn reload_apache(password: &str) {
    let _ = super::apache::reload(password).await;
}
