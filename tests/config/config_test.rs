use std::fs;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn get_env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

struct HomeGuard {
    original: Option<String>,
    _lock: Option<std::sync::MutexGuard<'static, ()>>,
}
impl HomeGuard {
    fn new(new_home: &std::path::Path) -> Self {
        let guard = get_env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let original = std::env::var("HOME").ok();
        let guard_static: std::sync::MutexGuard<'static, ()> =
            unsafe { std::mem::transmute(guard) };
        unsafe {
            std::env::set_var("HOME", new_home);
        }
        HomeGuard {
            original,
            _lock: Some(guard_static),
        }
    }
}
impl Drop for HomeGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(orig) = &self.original {
                std::env::set_var("HOME", orig);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }
}

fn with_config(toml: &str) -> (TempDir, devpanel::core::config::DevPanelConfig, HomeGuard) {
    let dir = TempDir::new().expect("tempdir");
    let cfg_dir = dir.path().join(".config").join("devpanel");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(cfg_dir.join("config.toml"), toml).unwrap();
    let guard = HomeGuard::new(dir.path());
    let config = devpanel::core::config::DevPanelConfig::load();
    (dir, config, guard)
}

#[test]
fn load_all_keys_present() {
    let toml = "devpanel_conf = \"/etc/apache2/sites-available/devpanel.conf\"\nhosts_file    = \"/etc/hosts\"\n";
    let (_dir, cfg, _guard) = with_config(toml);
    assert_eq!(
        cfg.devpanel_conf,
        "/etc/apache2/sites-available/devpanel.conf"
    );
    assert_eq!(cfg.hosts_file, "/etc/hosts");
}
#[test]
fn load_missing_file_uses_defaults() {
    let dir = TempDir::new().expect("tempdir");
    let _guard = HomeGuard::new(dir.path());
    let cfg = devpanel::core::config::DevPanelConfig::load();
    assert_eq!(
        cfg.devpanel_conf,
        "/etc/apache2/sites-available/devpanel.conf"
    );
    assert_eq!(cfg.hosts_file, "/etc/hosts");
}
#[test]
fn load_hosts_file_defaults_to_etc_hosts() {
    let toml = "devpanel_conf = \"/etc/apache2/sites-available/devpanel.conf\"\n";
    let (_dir, cfg, _guard) = with_config(toml);
    assert_eq!(cfg.hosts_file, "/etc/hosts");
}
#[test]
fn save_and_reload_is_identity() {
    let dir = TempDir::new().expect("tempdir");
    let _guard = HomeGuard::new(dir.path());
    let original = devpanel::core::config::DevPanelConfig {
        devpanel_conf: "/etc/apache2/sites-available/devpanel.conf".into(),
        hosts_file: "/etc/hosts".into(),
    };
    original.save();
    let reloaded = devpanel::core::config::DevPanelConfig::load();
    assert_eq!(reloaded.devpanel_conf, original.devpanel_conf);
}
