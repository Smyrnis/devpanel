//! Virtual host domain models and service functions.

mod certs;
pub mod config_text;
pub mod form;
pub mod model;
pub mod service;

pub use form::{FormMode, VHostForm};
pub use model::VHostEntry;
