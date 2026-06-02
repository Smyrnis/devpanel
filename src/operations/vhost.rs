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

pub async fn remove_host(password: &str, hostname: &str) -> Result<(), String> {
    super::rewrite_file_lines(password, crate::core::paths::HOSTS_FILE, |line| {
        !hosts_line_contains_hostname(line, hostname)
    })
    .await
}

fn hosts_line_contains_hostname(line: &str, hostname: &str) -> bool {
    let without_comment = line.split('#').next().unwrap_or("").trim();
    let mut parts = without_comment.split_whitespace();
    let _ip = parts.next();
    parts.any(|part| part == hostname)
}

pub async fn reload_apache(password: &str) {
    let _ = super::apache::reload(password).await;
}

#[cfg(test)]
mod tests {
    use super::hosts_line_contains_hostname;

    #[test]
    fn hosts_line_matching_ignores_comments() {
        assert!(!hosts_line_contains_hostname(
            "# 127.0.0.1 app.local",
            "app.local"
        ));
    }

    #[test]
    fn hosts_line_matching_matches_exact_hostname_column() {
        assert!(hosts_line_contains_hostname(
            "127.0.0.1 app.local www.app.local",
            "app.local"
        ));
        assert!(!hosts_line_contains_hostname(
            "127.0.0.1 myapp.local",
            "app.local"
        ));
    }
}
