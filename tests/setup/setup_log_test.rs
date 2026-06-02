use devpanel::core::setup_log::{LogLevel, SetupLogEntry, format_setup_log_line};

fn parse(line: &str) -> Option<SetupLogEntry> {
    SetupLogEntry::parse(line)
}

#[test]
fn parse_ok_level() {
    let e = parse("2025-01-15 14:32:01 [OK]    Apache reloaded successfully").unwrap();
    assert_eq!(e.level, LogLevel::Ok);
    assert_eq!(e.message, "Apache reloaded successfully");
}
#[test]
fn parse_step_level() {
    let e = parse("2025-01-15 14:30:00 [STEP]  Starting DevPanel setup").unwrap();
    assert_eq!(e.level, LogLevel::Step);
}
#[test]
fn parse_info_level() {
    let e = parse("2025-01-15 14:30:05 [INFO]  User: johndoe").unwrap();
    assert_eq!(e.level, LogLevel::Info);
    assert_eq!(e.message, "User: johndoe");
}
#[test]
fn parse_warn_level() {
    let e = parse("2025-01-15 14:31:00 [WARN]  mod_php not found for PHP 5.6").unwrap();
    assert_eq!(e.level, LogLevel::Warn);
}
#[test]
fn parse_error_level() {
    let e = parse("2025-01-15 14:31:59 [ERROR] Package install failed: dpkg error").unwrap();
    assert_eq!(e.level, LogLevel::Error);
}
#[test]
fn parse_cmd_level() {
    let e = parse("2025-01-15 14:30:10 [CMD]   apt-get update").unwrap();
    assert_eq!(e.level, LogLevel::Cmd);
}
#[test]
fn parse_out_level() {
    let e =
        parse("2025-01-15 14:30:15 [OUT]   Hit:1 http://archive.ubuntu.com/ubuntu jammy InRelease")
            .unwrap();
    assert_eq!(e.level, LogLevel::Out);
}
#[test]
fn parse_postinst_level() {
    let e = parse("2025-01-15 14:29:55 [POSTINST] postinst triggered — action: configure").unwrap();
    assert_eq!(e.level, LogLevel::PostInst);
}
#[test]
fn parse_unknown_level_tag() {
    let e = parse("2025-01-15 14:31:30 [WEIRD] something unusual").unwrap();
    assert_eq!(e.level, LogLevel::Unknown);
    assert_eq!(e.message, "something unusual");
}
#[test]
fn parse_no_bracket_returns_unknown() {
    let e = parse("2025-01-15 14:31:30 plain text without bracket").unwrap();
    assert_eq!(e.level, LogLevel::Unknown);
}
#[test]
fn parse_empty_string_returns_none() {
    assert!(parse("").is_none());
}
#[test]
fn parse_whitespace_only_returns_none() {
    assert!(parse("   ").is_none());
}
#[test]
fn parse_line_too_short_returns_none() {
    assert!(parse("2025-01-15 14:30").is_none());
}
#[test]
fn parse_preserves_timestamp_exactly() {
    let e = parse("2026-03-31 23:59:59 [OK]    done").unwrap();
    assert_eq!(e.timestamp, "2026-03-31 23:59:59");
}
#[test]
fn parse_message_with_colon() {
    let e = parse(
        "2025-06-01 10:00:00 [INFO]  devpanel_conf: /etc/apache2/sites-available/devpanel.conf",
    )
    .unwrap();
    assert_eq!(
        e.message,
        "devpanel_conf: /etc/apache2/sites-available/devpanel.conf"
    );
}
#[test]
fn parse_message_trimmed_of_whitespace() {
    let e = parse("2025-06-01 10:00:00 [OK]      extra spaces before message   ").unwrap();
    assert!(!e.message.starts_with(' '));
    assert!(!e.message.ends_with(' '));
}
#[test]
fn only_error_and_warn_are_issues() {
    let lines = [
        "2025-01-15 14:30:00 [STEP]  Installing packages",
        "2025-01-15 14:30:05 [OK]    Done",
        "2025-01-15 14:30:10 [WARN]  PHP 5.6 not found",
        "2025-01-15 14:30:15 [INFO]  Skipping mod",
        "2025-01-15 14:30:20 [ERROR] apt-get failed",
    ];
    let issues: Vec<SetupLogEntry> = lines
        .iter()
        .filter_map(|l| SetupLogEntry::parse(l))
        .filter(|e| matches!(e.level, LogLevel::Error | LogLevel::Warn))
        .collect();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].level, LogLevel::Warn);
    assert_eq!(issues[1].level, LogLevel::Error);
}

#[test]
fn format_setup_log_line_matches_shell_logger_spacing() {
    let line = format_setup_log_line(
        "2026-05-24 10:11:12",
        LogLevel::Step,
        "Starting in-app first-run setup",
    );

    assert_eq!(
        line,
        "2026-05-24 10:11:12 [STEP ] Starting in-app first-run setup"
    );
    let parsed = SetupLogEntry::parse(&line).unwrap();
    assert_eq!(parsed.level, LogLevel::Step);
    assert_eq!(parsed.message, "Starting in-app first-run setup");
}
