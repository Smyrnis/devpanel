//! Core application data, configuration, and durable support.
//!
//! `core` owns durable configuration, database access, paths, setup state,
//! error types, dry-run support, and runtime theme loading. Host OS
//! integrations belong under `infra`, UI rendering belongs under `ui`, and
//! privileged command wrappers belong under `operations`.

pub mod app_config;
pub mod config;
pub mod db;
pub mod dry_run;
pub mod error;
pub mod first_run;
pub mod paths;
pub mod setup_log;
pub mod theme;
