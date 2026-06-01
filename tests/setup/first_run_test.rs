use devpanel::installer::{FirstRunInstallOptions, FirstRunPackageStatus, service};
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

struct HomeGuard {
    original: Option<String>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl HomeGuard {
    fn new(home: &std::path::Path) -> Self {
        let guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let original = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", home);
        }
        Self {
            original,
            _lock: guard,
        }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(original) = &self.original {
                std::env::set_var("HOME", original);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }
}

#[test]
fn selected_packages_default_skips_all_optional_packages() {
    let packages = service::selected_packages(FirstRunInstallOptions::default());

    assert!(packages.is_empty());
}

#[test]
fn php_version_options_include_legacy_and_latest_versions() {
    assert_eq!(
        devpanel::domain::tools::model::PHP_VERSION_OPTIONS,
        &[
            "5.6", "7.0", "7.1", "7.2", "7.3", "7.4", "8.0", "8.1", "8.2", "8.3", "8.4", "8.5",
        ]
    );
}

#[test]
fn selected_packages_can_include_apache_and_php() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: true,
        install_php: true,
        install_mysql: false,
        install_php_extras: false,
    });

    assert!(packages.contains(&"apache2"));
    assert!(packages.contains(&"php8.5"));
    assert!(packages.contains(&"php8.5-cli"));
    assert!(packages.contains(&"libapache2-mod-php8.5"));
}

#[test]
fn selected_packages_can_include_apache_only() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: true,
        install_php: false,
        install_mysql: false,
        install_php_extras: false,
    });

    assert_eq!(packages, vec!["apache2"]);
}

#[test]
fn selected_packages_can_include_php_only() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: false,
        install_php: true,
        install_mysql: false,
        install_php_extras: false,
    });

    assert_eq!(
        packages,
        vec![
            "php8.5",
            "php8.5-cli",
            "php8.5-common",
            "libapache2-mod-php8.5"
        ]
    );
}

#[test]
fn selected_packages_can_include_every_optional_group() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: true,
        install_php: true,
        install_mysql: true,
        install_php_extras: true,
    });

    assert_eq!(
        packages,
        vec![
            "apache2",
            "php8.5",
            "php8.5-cli",
            "php8.5-common",
            "libapache2-mod-php8.5",
            "php8.5-mysql",
            "php8.5-xml",
            "php8.5-mbstring",
            "mysql-server",
        ]
    );
}

#[test]
fn selected_packages_can_skip_mysql() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: false,
        install_php: false,
        install_mysql: false,
        install_php_extras: true,
    });

    assert!(!packages.contains(&"mysql-server"));
    assert!(packages.contains(&"php8.5-mysql"));
}

#[test]
fn selected_packages_can_skip_php_extras() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: false,
        install_php: false,
        install_mysql: true,
        install_php_extras: false,
    });

    assert!(packages.contains(&"mysql-server"));
    assert!(!packages.contains(&"php8.5-mysql"));
    assert!(!packages.contains(&"php8.5-xml"));
    assert!(!packages.contains(&"php8.5-mbstring"));
}

#[tokio::test]
async fn dry_run_status_scan_returns_synthetic_not_installed_status() {
    assert!(devpanel::core::dry_run::active());

    let status = service::scan_first_run_status().await;

    assert_eq!(status.projects_dir, FirstRunPackageStatus::NotInstalled);
    assert_eq!(status.apache, FirstRunPackageStatus::NotInstalled);
    assert_eq!(status.php, FirstRunPackageStatus::NotInstalled);
    assert_eq!(status.mysql, FirstRunPackageStatus::NotInstalled);
    assert_eq!(status.php_extras, FirstRunPackageStatus::NotInstalled);
}

#[tokio::test]
async fn dry_run_install_returns_preview_without_real_operations() {
    assert!(devpanel::core::dry_run::active());

    let result = service::run_first_run_install(
        "local-dev-password".to_string(),
        FirstRunInstallOptions {
            install_apache: false,
            install_php: false,
            install_mysql: false,
            install_php_extras: false,
        },
    )
    .await
    .expect("dry-run install should return preview");

    assert!(result.contains("[dry-run] Would create the projects directory"));
    assert!(result.contains("no optional packages selected"));
    assert!(!result.contains("apache2"));
    assert!(!result.contains("php8.5"));
    assert!(!result.contains("mysql-server"));
    assert!(!result.contains("php8.5-mysql"));
}

#[test]
fn first_run_sentinel_uses_home_directory() {
    let temp = TempDir::new().expect("tempdir");
    let _guard = HomeGuard::new(temp.path());

    assert!(devpanel::core::first_run::is_first_run());

    devpanel::core::first_run::mark_done();

    assert!(!devpanel::core::first_run::is_first_run());
    assert!(
        temp.path()
            .join(".config")
            .join("devpanel")
            .join("first_run_done")
            .exists()
    );
}
