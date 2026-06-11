use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    home: Option<String>,
    user: Option<String>,
    sudo_user: Option<String>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn new() -> Self {
        let guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let home = std::env::var("HOME").ok();
        let user = std::env::var("USER").ok();
        let sudo_user = std::env::var("SUDO_USER").ok();
        unsafe {
            std::env::set_var("HOME", "/home/devpanel-test");
            std::env::set_var("USER", "devpanel-test");
            std::env::set_var("SUDO_USER", "devpanel-test");
        }
        Self {
            home,
            user,
            sudo_user,
            _lock: guard,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        restore_env("HOME", self.home.as_deref());
        restore_env("USER", self.user.as_deref());
        restore_env("SUDO_USER", self.sudo_user.as_deref());
    }
}

fn restore_env(key: &str, value: Option<&str>) {
    unsafe {
        if let Some(value) = value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }
}

#[test]
fn composer_self_update_args_support_major_versions() {
    assert_eq!(
        devpanel::operations::tools::composer_self_update_args("latest"),
        vec!["composer", "self-update"]
    );
    assert_eq!(
        devpanel::operations::tools::composer_self_update_args("2"),
        vec!["composer", "self-update", "--2"]
    );
    assert_eq!(
        devpanel::operations::tools::composer_self_update_args("rollback"),
        vec!["composer", "self-update", "--rollback"]
    );
    assert_eq!(
        devpanel::operations::tools::composer_self_update_args("2.8.8"),
        vec!["composer", "self-update", "2.8.8"]
    );
}

#[test]
fn composer_installer_is_verified_before_execution() {
    let script = devpanel::operations::tools::composer_install_script();

    assert!(script.contains("https://composer.github.io/installer.sig"));
    assert!(script.contains("hash_file('sha384'"));
    assert!(script.contains("Invalid Composer installer checksum"));
    assert!(script.contains("php /tmp/composer-setup.php"));
    assert!(
        std::process::Command::new("sh")
            .args(["-n", "-c", &script])
            .status()
            .expect("shell should parse Composer installer command")
            .success()
    );
}

#[test]
fn nvm_runtime_probe_sources_the_user_installation() {
    let script = devpanel::operations::tools::nvm_runtime_probe_script("node --version");

    assert!(script.contains("export NVM_DIR=\"$HOME/.nvm\""));
    assert!(script.contains(". \"$NVM_DIR/nvm.sh\""));
    assert!(script.ends_with("node --version"));
}

#[tokio::test]
async fn dry_run_node_install_uses_target_user_and_sources_nvm() {
    let _guard = EnvGuard::new();

    let output = devpanel::operations::tools::install_node_version("", "22")
        .await
        .expect("dry-run Node install should return a command preview");

    assert!(output.contains("-u devpanel-test"));
    assert!(output.contains("HOME=/home/devpanel-test"));
    assert!(output.contains(". \"$NVM_DIR/nvm.sh\""));
    assert!(output.contains("nvm install '22'"));
}

#[tokio::test]
async fn dry_run_node_switch_updates_default_and_active_versions() {
    let _guard = EnvGuard::new();

    let output = devpanel::operations::tools::switch_node_version("", "22")
        .await
        .expect("dry-run Node switch should return a command preview");

    assert!(output.contains("nvm alias default '22'"));
    assert!(output.contains("nvm use '22'"));
}

#[tokio::test]
async fn composer_sudo_completion_targets_dashboard() {
    use devpanel::infra::sudo_prompt::{ComposerCommand, SudoCommand};
    use devpanel::messages::{DashboardMessage, Message};

    let message = Box::new(ComposerCommand { update: true }).execute("").await;

    assert!(matches!(
        message,
        Message::Dashboard(DashboardMessage::ComposerDone(true, _))
    ));
}

#[tokio::test]
async fn node_sudo_completion_targets_dashboard() {
    use devpanel::infra::sudo_prompt::{NodeNvmAction, NodeNvmCommand, SudoCommand};
    use devpanel::messages::{DashboardMessage, Message};

    let _guard = EnvGuard::new();
    let message = Box::new(NodeNvmCommand {
        action: NodeNvmAction::SwitchNode("22".to_string()),
    })
    .execute("")
    .await;

    assert!(matches!(
        message,
        Message::Dashboard(DashboardMessage::NodeNvmDone(true, _))
    ));
}
