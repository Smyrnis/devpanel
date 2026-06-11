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
        devpanel::core::app_config::php_version_numbers(),
        vec![
            "5.6", "7.0", "7.1", "7.2", "7.3", "7.4", "8.0", "8.1", "8.2", "8.3", "8.4", "8.5"
        ]
    );
}

#[test]
fn runtime_version_options_include_composer_and_node_defaults() {
    assert_eq!(
        devpanel::core::app_config::default_composer_version(),
        "latest"
    );
    assert_eq!(
        devpanel::core::app_config::composer_versions(),
        vec![
            "latest".to_string(),
            "2".to_string(),
            "rollback".to_string()
        ]
    );
    assert_eq!(devpanel::core::app_config::default_node_version(), "22");
    assert_eq!(
        devpanel::core::app_config::node_versions(),
        vec![
            "20".to_string(),
            "22".to_string(),
            "24".to_string(),
            "node".to_string()
        ]
    );

    let dashboard = devpanel::ui::tabs::dashboard::DashboardTab::new();
    assert_eq!(dashboard.selected_composer_version, "latest");
    assert_eq!(dashboard.selected_node_version, "22");
}

#[test]
fn dashboard_applies_runtime_scan_results() {
    let mut dashboard = devpanel::ui::tabs::dashboard::DashboardTab::new();
    dashboard.runtimes_scanning = true;
    dashboard.apply_runtime_scan(devpanel::domain::tools::InstalledTools {
        composer_version: Some("Composer version 2.8.9".to_string()),
        node_version: Some("v22.0.0".to_string()),
        npm_version: Some("10.0.0".to_string()),
        nvm_available: true,
        redis_installed: false,
        redis_running: false,
        redis_memory: None,
    });

    assert_eq!(
        dashboard.installed_tools.node_version.as_deref(),
        Some("v22.0.0")
    );
    assert!(dashboard.installed_tools.nvm_available);
    assert!(!dashboard.runtimes_scanning);
}

#[test]
fn selected_packages_can_include_apache_and_php() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: true,
        install_php: true,
        install_mysql: false,
        install_php_extras: false,
        install_composer: false,
        install_node_nvm: false,
    });

    assert!(packages.contains(&"apache2".to_string()));
    assert!(packages.contains(&"php8.5".to_string()));
    assert!(packages.contains(&"php8.5-cli".to_string()));
    assert!(packages.contains(&"php8.5-fpm".to_string()));
}

#[test]
fn selected_packages_can_include_apache_only() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: true,
        install_php: false,
        install_mysql: false,
        install_php_extras: false,
        install_composer: false,
        install_node_nvm: false,
    });

    assert_eq!(packages, vec!["apache2".to_string()]);
}

#[test]
fn selected_packages_can_include_php_only() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: false,
        install_php: true,
        install_mysql: false,
        install_php_extras: false,
        install_composer: false,
        install_node_nvm: false,
    });

    assert_eq!(
        packages,
        vec![
            "php8.5".to_string(),
            "php8.5-cli".to_string(),
            "php8.5-common".to_string(),
            "php8.5-fpm".to_string(),
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
        install_composer: false,
        install_node_nvm: false,
    });

    assert_eq!(
        packages,
        vec![
            "apache2".to_string(),
            "php8.5".to_string(),
            "php8.5-cli".to_string(),
            "php8.5-common".to_string(),
            "php8.5-fpm".to_string(),
            "php8.5-mysql".to_string(),
            "php8.5-xml".to_string(),
            "php8.5-mbstring".to_string(),
            "mysql-server".to_string(),
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
        install_composer: false,
        install_node_nvm: false,
    });

    assert!(!packages.contains(&"mysql-server".to_string()));
    assert!(packages.contains(&"php8.5-mysql".to_string()));
}

#[test]
fn selected_packages_can_skip_php_extras() {
    let packages = service::selected_packages(FirstRunInstallOptions {
        install_apache: false,
        install_php: false,
        install_mysql: true,
        install_php_extras: false,
        install_composer: false,
        install_node_nvm: false,
    });

    assert!(packages.contains(&"mysql-server".to_string()));
    assert!(!packages.contains(&"php8.5-mysql".to_string()));
    assert!(!packages.contains(&"php8.5-xml".to_string()));
    assert!(!packages.contains(&"php8.5-mbstring".to_string()));
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
    assert_eq!(status.composer, FirstRunPackageStatus::NotInstalled);
    assert_eq!(status.node_nvm, FirstRunPackageStatus::NotInstalled);
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
            install_composer: false,
            install_node_nvm: false,
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
fn composer_and_node_nvm_are_not_selected_as_apt_packages() {
    let options = FirstRunInstallOptions {
        install_apache: false,
        install_php: false,
        install_mysql: false,
        install_php_extras: false,
        install_composer: true,
        install_node_nvm: true,
    };

    let packages = service::selected_packages(options);
    let prerequisites = service::selected_runtime_prerequisite_packages(options);

    assert!(packages.is_empty());
    assert_eq!(
        prerequisites,
        vec![
            "curl".to_string(),
            "ca-certificates".to_string(),
            "php-cli".to_string()
        ]
    );
}

#[test]
fn composer_does_not_duplicate_php_cli_when_php_is_selected() {
    let options = FirstRunInstallOptions {
        install_apache: false,
        install_php: true,
        install_mysql: false,
        install_php_extras: false,
        install_composer: true,
        install_node_nvm: false,
    };

    let prerequisites = service::selected_runtime_prerequisite_packages(options);

    assert_eq!(
        prerequisites,
        vec!["curl".to_string(), "ca-certificates".to_string()]
    );
}

#[tokio::test]
async fn dry_run_install_preview_includes_runtime_tool_selections() {
    assert!(devpanel::core::dry_run::active());

    let result = service::run_first_run_install(
        "local-dev-password".to_string(),
        FirstRunInstallOptions {
            install_apache: false,
            install_php: false,
            install_mysql: false,
            install_php_extras: false,
            install_composer: true,
            install_node_nvm: true,
        },
    )
    .await
    .expect("dry-run install should return preview");

    assert!(result.contains("Runtime tools: Composer latest, Node 22 via NVM"));
    assert!(!result.contains("composer installer"));
    assert!(!result.contains("nvm"));
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

#[test]
fn package_setup_marks_first_run_done_after_full_setup() {
    let install_tools =
        std::fs::read_to_string("installation/dependencies/install_tools.sh").unwrap();
    let full_setup = std::fs::read_to_string("installation/devpanel-setup.sh").unwrap();
    let projects_only =
        std::fs::read_to_string("installation/devpanel-create-projects-dir.sh").unwrap();

    assert!(install_tools.contains("mark_first_run_done()"));
    assert!(install_tools.contains("first_run_done"));
    assert!(install_tools.contains("setfacl -m \"u:www-data:--x\""));
    assert!(install_tools.contains("setfacl -R -m \"u:www-data:rX\""));
    assert!(install_tools.contains("d:u:www-data:rX"));
    assert!(full_setup.contains("mark_first_run_done"));
    assert!(full_setup.contains("install_composer_if_requested"));
    assert!(full_setup.contains("install_node_nvm_if_requested"));
    assert!(!projects_only.contains("mark_first_run_done"));
}
