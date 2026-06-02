pub use crate::installer::{FirstRunPackage, FirstRunSetupStatus};

#[derive(Debug, Clone)]
pub enum FirstRunMessage {
    Continue,
    Exit,
    TogglePackage(FirstRunPackage),
    ToggleApache(bool),
    TogglePhp(bool),
    ToggleMysql(bool),
    TogglePhpExtras(bool),
    ScanStatus,
    StatusScanned(FirstRunSetupStatus),
    ProgressTick,
    LogLoaded(Vec<String>),
    InstallDone(bool, String),
}
