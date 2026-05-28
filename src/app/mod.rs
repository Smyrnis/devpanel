//! Application orchestration.
//!
//! This module wires together top-level app state, shell rendering,
//! subscriptions, notifications, update routing, and domain handlers.

mod handlers;
mod notifications;
mod shell;
pub mod state;
mod subscriptions;
pub mod update;

pub use state::{App, Toast};
