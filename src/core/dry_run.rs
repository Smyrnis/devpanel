#[inline(always)]
pub const fn active() -> bool {
    !cfg!(feature = "production")
}

#[inline(always)]
#[allow(dead_code)]
pub const fn is_production() -> bool {
    cfg!(feature = "production")
}

#[allow(dead_code)]
pub fn mode_label() -> &'static str {
    if active() {
        "DEV (dry-run)"
    } else {
        "PRODUCTION"
    }
}

pub fn ok(cmd: &str) -> Result<String, String> {
    Ok(format!("[dry-run] would run: {}", cmd))
}
#[allow(dead_code)]
pub fn success(what: &str) -> (bool, String) {
    (true, format!("[dry-run] skipped: {}", what))
}

pub fn log(what: &str) {
    eprintln!("[DevPanel dry-run] {}", what);
}
