//! Standalone first-run installer boundary.
//!
//! The first-run window and setup workflow are kept under `installation/first_run`
//! so the normal application UI and domain modules do not own installer logic.

#[path = "../installation/first_run/model.rs"]
pub mod model;
#[path = "../installation/first_run/service.rs"]
pub mod service;
#[path = "../installation/first_run/window/mod.rs"]
pub mod window;

pub use model::{
    FirstRunInstallOptions, FirstRunPackage, FirstRunPackageStatus, FirstRunSetupStatus,
};
