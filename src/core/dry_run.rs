#[inline(always)]
pub const fn active() -> bool {
    !cfg!(feature = "production")
}

#[inline(always)]
pub const fn is_production() -> bool {
    cfg!(feature = "production")
}

pub fn mode_label() -> &'static str {
    if active() {
        "DEV (dry-run)"
    } else {
        "PRODUCTION"
    }
}

pub fn log(what: &str) {
    eprintln!("[DevPanel dry-run] {}", what);
}
