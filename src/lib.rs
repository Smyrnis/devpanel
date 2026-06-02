#![allow(dead_code)]
pub mod core;
pub mod domain;
pub mod helpers;
pub mod infra;
pub mod installer;
pub mod lang;
pub mod messages;
pub mod operations;
pub mod ui;

pub use installer::window as install_window;
pub use ui::tabs;
pub use ui::templates;
