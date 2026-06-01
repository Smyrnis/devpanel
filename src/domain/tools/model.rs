#[derive(Debug, Clone, PartialEq)]
pub enum PhpStatus {
    Installed,
    Available,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PhpRelease {
    pub version: String,
    pub status: PhpStatus,
    pub is_active: bool,
    pub apache_mod_available: bool,
    pub apache_mod_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ApacheModule {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PhpExtension {
    pub name: String,
    pub pkg_suffix: String,
    pub installed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolSection {
    Php,
    ApacheMods,
    PhpExts,
    Runtimes,
    Database,
}

#[derive(Debug, Clone, Default)]
pub struct InstalledTools {
    pub composer_version: Option<String>,
    pub node_version: Option<String>,
    pub npm_version: Option<String>,
    pub nvm_available: bool,
    pub redis_installed: bool,
    pub redis_running: bool,
    pub redis_memory: Option<String>,
}

pub const PHP_VERSION_OPTIONS: &[&str] = &[
    "5.6", "7.0", "7.1", "7.2", "7.3", "7.4", "8.0", "8.1", "8.2", "8.3", "8.4", "8.5",
];

pub fn default_php_releases() -> Vec<PhpRelease> {
    PHP_VERSION_OPTIONS
        .iter()
        .copied()
        .map(|version| PhpRelease {
            version: version.into(),
            status: PhpStatus::Unknown,
            is_active: false,
            apache_mod_available: false,
            apache_mod_enabled: false,
        })
        .collect()
}

pub fn default_php_extensions() -> Vec<PhpExtension> {
    [
        ("curl", "php-curl"),
        ("gd", "php-gd"),
        ("mbstring", "php-mbstring"),
        ("xml", "php-xml"),
        ("zip", "php-zip"),
        ("mysql", "php-mysql"),
        ("pgsql", "php-pgsql"),
        ("redis", "php-redis"),
        ("intl", "php-intl"),
        ("bcmath", "php-bcmath"),
        ("soap", "php-soap"),
        ("imagick", "php-imagick"),
        ("xdebug", "php-xdebug"),
        ("sqlite3", "php-sqlite3"),
    ]
    .into_iter()
    .map(|(name, pkg_suffix)| PhpExtension {
        name: name.into(),
        pkg_suffix: pkg_suffix.into(),
        installed: false,
    })
    .collect()
}
