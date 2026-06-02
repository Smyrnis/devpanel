//! Domain models and behavior that should not depend on UI rendering.
//!
//! UI tabs can keep view state here only by reference or re-export, while parsing,
//! config generation, and durable domain models live in these modules.

pub mod dashboard;
pub mod settings;
pub mod ssh_keys;
pub mod tools;
pub mod vhosts;
