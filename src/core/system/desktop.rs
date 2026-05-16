use crate::core::paths;
use std::path::PathBuf;

pub fn get_home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
}

pub fn xdg_open(path: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

pub fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

pub fn open_php_ini(active_php: &Option<String>) -> std::io::Result<()> {
    if let Some(version) = active_php {
        let short = version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let cli_ini = paths::php_cli_ini(&short);
        let apache_ini = paths::php_apache_ini(&short);
        if std::path::Path::new(&cli_ini).exists() {
            return xdg_open(&cli_ini);
        }
        if std::path::Path::new(&apache_ini).exists() {
            return xdg_open(&apache_ini);
        }
    }
    xdg_open(paths::PHP_ETC_DIR)
}
