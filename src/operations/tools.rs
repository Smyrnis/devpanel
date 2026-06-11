use crate::core::paths;
use crate::infra::system::shell_quote;

const NVM_INSTALL_VERSION: &str = "v0.40.1";

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
        return switch_composer_version(password, "latest").await;
    }

    install_composer(password, None).await
}

pub async fn install_composer(password: &str, version: Option<&str>) -> Result<String, String> {
    let install_cmd = composer_install_script();
    super::run(password, &["sh", "-c", &install_cmd]).await?;

    if let Some(version) = version
        && version != "latest"
    {
        switch_composer_version(password, version).await?;
    }

    Ok("Composer installed globally".to_string())
}

pub fn composer_install_script() -> String {
    format!(
        "set -e; \
         trap 'rm -f /tmp/composer-setup.php' EXIT; \
         expected_checksum=\"$(php -r 'copy(\"https://composer.github.io/installer.sig\", \"php://stdout\");')\"; \
         php -r \"copy('https://getcomposer.org/installer', '/tmp/composer-setup.php');\"; \
         actual_checksum=\"$(php -r \"echo hash_file('sha384', '/tmp/composer-setup.php');\")\"; \
         if [ \"$expected_checksum\" != \"$actual_checksum\" ]; then \
             echo 'ERROR: Invalid Composer installer checksum' >&2; \
             exit 1; \
         fi; \
         php /tmp/composer-setup.php --install-dir={} --filename=composer",
        paths::COMPOSER_INSTALL_DIR,
    )
}

pub async fn switch_composer_version(password: &str, version: &str) -> Result<String, String> {
    let args = composer_self_update_args(version);
    super::run(password, &args).await?;
    Ok(format!("Composer {} selected", version_label(version)))
}

pub fn composer_self_update_args(version: &str) -> Vec<&str> {
    match version {
        "latest" => vec!["composer", "self-update"],
        "1" => vec!["composer", "self-update", "--1"],
        "2" => vec!["composer", "self-update", "--2"],
        "rollback" => vec!["composer", "self-update", "--rollback"],
        version => vec!["composer", "self-update", version],
    }
}

pub async fn install_nvm(password: &str) -> Result<String, String> {
    run_nvm_user_script(password, nvm_install_script()).await
}

pub async fn install_nvm_and_node(password: &str, version: &str) -> Result<String, String> {
    let version_arg = shell_quote(version);
    let script = format!(
        "{} && . \"$NVM_DIR/nvm.sh\" && nvm install {} && nvm alias default {} && nvm use {}",
        nvm_install_script(),
        version_arg,
        version_arg,
        version_arg
    );
    run_nvm_user_script(password, script).await
}

pub async fn install_node_version(password: &str, version: &str) -> Result<String, String> {
    run_existing_nvm_command(password, &format!("nvm install {}", shell_quote(version))).await
}

pub async fn switch_node_version(password: &str, version: &str) -> Result<String, String> {
    let version = shell_quote(version);
    run_existing_nvm_command(
        password,
        &format!("nvm alias default {version} && nvm use {version}"),
    )
    .await
}

pub async fn set_default_node_version(password: &str, version: &str) -> Result<String, String> {
    run_existing_nvm_command(
        password,
        &format!("nvm alias default {}", shell_quote(version)),
    )
    .await
}

pub fn nvm_runtime_probe_script(command: &str) -> String {
    format!("export NVM_DIR=\"$HOME/.nvm\"; . \"$NVM_DIR/nvm.sh\" && {command}")
}

fn nvm_install_script() -> String {
    format!(
        "export NVM_DIR=\"$HOME/.nvm\"; if [ ! -s \"$NVM_DIR/nvm.sh\" ]; then curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/{}/install.sh | bash; fi",
        NVM_INSTALL_VERSION
    )
}

async fn run_existing_nvm_command(password: &str, command: &str) -> Result<String, String> {
    let script = nvm_runtime_probe_script(command);
    run_nvm_user_script(password, script).await
}

async fn run_nvm_user_script(password: &str, script: String) -> Result<String, String> {
    let user =
        target_user().ok_or_else(|| "Could not determine target user for NVM".to_string())?;
    let home = target_home(&user).ok_or_else(|| {
        format!(
            "Could not determine home directory for target user {}",
            user
        )
    })?;
    let home_env = format!("HOME={home}");
    super::run(
        password,
        &[
            "-u",
            user.as_str(),
            "env",
            home_env.as_str(),
            "bash",
            "-lc",
            &script,
        ],
    )
    .await
}

fn target_user() -> Option<String> {
    ["SUDO_USER", "USER", "LOGNAME"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.trim().is_empty() && value != "root")
}

fn target_home(user: &str) -> Option<String> {
    if let Ok(home) = std::env::var("HOME")
        && !home.trim().is_empty()
        && home != "/root"
    {
        return Some(home);
    }

    std::fs::read_to_string("/etc/passwd")
        .ok()?
        .lines()
        .find_map(|line| {
            let parts = line.split(':').collect::<Vec<_>>();
            (parts.len() >= 6 && parts[0] == user).then(|| parts[5].to_string())
        })
}

fn version_label(version: &str) -> &str {
    if version == "latest" {
        "latest"
    } else {
        version
    }
}

pub async fn redis_service_op(
    password: &str,
    action: &str,
    service: &str,
) -> Result<String, String> {
    super::systemctl(password, action, service).await
}
