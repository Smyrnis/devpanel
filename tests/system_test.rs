use devpanel::core::system::shell_quote;

struct HomeGuard {
    original: Option<String>,
}

impl HomeGuard {
    fn new(new_home: &str) -> Self {
        let original = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", new_home);
        }
        HomeGuard { original }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(orig) = &self.original {
                std::env::set_var("HOME", orig);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }
}

#[test]
fn shell_quote_simple_path() {
    assert_eq!(shell_quote("/var/www/html"), "'/var/www/html'");
}

#[test]
fn shell_quote_path_with_spaces() {
    assert_eq!(
        shell_quote("/home/user/my project"),
        "'/home/user/my project'"
    );
}

#[test]
fn shell_quote_path_with_single_quote() {
    // A single quote inside the string must be escaped as '\''
    let result = shell_quote("/home/user/can't");
    assert_eq!(result, "'/home/user/can'\\''t'");
}

#[test]
fn shell_quote_empty_string() {
    assert_eq!(shell_quote(""), "''");
}

#[test]
fn shell_quote_path_with_special_chars() {
    // Dollar signs, stars, etc. are safe inside single quotes in bash.
    let result = shell_quote("/var/www/$special*path");
    assert_eq!(result, "'/var/www/$special*path'");
}

#[test]
fn shell_quote_multiple_single_quotes() {
    let result = shell_quote("a'b'c");
    assert_eq!(result, "'a'\\''b'\\''c'");
}

#[test]
fn get_home_returns_non_empty_path() {
    let home = devpanel::core::system::get_home();
    assert!(!home.as_os_str().is_empty());
}

#[test]
fn get_home_reflects_home_env_var() {
    let _guard = HomeGuard::new("/tmp/test_home_value");
    let home = devpanel::core::system::get_home();
    assert_eq!(home.to_str().unwrap(), "/tmp/test_home_value");
}
