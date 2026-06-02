//! Infrastructure integrations for the host system and runtime environment.
//!
//! Domain and UI code should use these modules for OS-facing behavior such as
//! opening files, shell quoting, service helpers, clipboard access, and file
//! watching.

pub mod file_watcher;
pub mod sudo_prompt;
pub mod system;
