//! User interface modules.
//!
//! `ui` owns visual layout, tab views, reusable templates, icons, and
//! presentation utilities. Domain data and system actions should stay in
//! `core` or `operations` and be passed into views through tab state.

pub mod components;
pub mod icons;
pub mod layout;
pub mod tabs;
pub mod templates;
pub mod utils;
