// src/dry_run.rs

/// Returns `true` when running in dev / dry-run mode.
/// Resolved entirely at compile time — zero runtime cost.
#[inline(always)]
pub const fn active() -> bool {
    !cfg!(feature = "production")
}

/// Returns `true` when running with the `production` feature enabled.
#[inline(always)]
pub const fn is_production() -> bool {
    cfg!(feature = "production")
}

/// Build a human-readable label for the current mode.
pub fn mode_label() -> &'static str {
    if active() { "DEV (dry-run)" } else { "PRODUCTION" }
}

/// Return a fake `Ok` result for a command that was skipped.
/// The string explains what *would* have happened.
pub fn ok(cmd: &str) -> Result<String, String> {
    Ok(format!("[dry-run] would run: {}", cmd))
}

/// Return a fake success `(true, msg)` tuple (used by tasks that return `(bool, String)`).
pub fn success(what: &str) -> (bool, String) {
    (true, format!("[dry-run] skipped: {}", what))
}

/// Log a dry-run skip to stderr (visible in `cargo run` terminal).
pub fn log(what: &str) {
    eprintln!("[DevPanel dry-run] {}", what);
}
