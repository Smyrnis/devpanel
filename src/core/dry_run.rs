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

pub fn log(what: &str) {
    eprintln!("[DevPanel dry-run] {}", what);
}

pub fn log_user_action(action: &str) {
    log(&format!("user action: {action}"));
}

pub fn preview_command(program: &str, args: &[&str]) -> String {
    let suffix = if args.is_empty() {
        String::new()
    } else {
        format!(" {}", args.join(" "))
    };
    format!("[dry-run] would run: {program}{suffix}")
}
