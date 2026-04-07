use devpanel::core::setup_log::{LogLevel, SetupLogEntry};

fn parse(line: &str) -> Option<SetupLogEntry> {
    SetupLogEntry::parse(line)
}

#[test]
fn parse_ok_level() {
    let line = "2025-01-15 14:32:01 [OK]    Apache reloaded successfully";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.timestamp, "2025-01-15 14:32:01");
    assert_eq!(entry.level, LogLevel::Ok);
    assert_eq!(entry.message, "Apache reloaded successfully");
}

#[test]
fn parse_step_level() {
    let line = "2025-01-15 14:30:00 [STEP]  Starting DevPanel setup";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Step);
    assert_eq!(entry.message, "Starting DevPanel setup");
}

#[test]
fn parse_info_level() {
    let line = "2025-01-15 14:30:05 [INFO]  User: johndoe";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.message, "User: johndoe");
}

#[test]
fn parse_warn_level() {
    let line = "2025-01-15 14:31:00 [WARN]  mod_php not found for PHP 5.6";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Warn);
    assert!(entry.message.contains("mod_php not found"));
}

#[test]
fn parse_error_level() {
    let line = "2025-01-15 14:31:59 [ERROR] Package install failed: dpkg error";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Error);
    assert!(entry.message.contains("Package install failed"));
}

#[test]
fn parse_cmd_level() {
    let line = "2025-01-15 14:30:10 [CMD]   apt-get update";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Cmd);
}

#[test]
fn parse_out_level() {
    let line = "2025-01-15 14:30:15 [OUT]   Hit:1 http://archive.ubuntu.com/ubuntu jammy InRelease";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Out);
}

#[test]
fn parse_postinst_level() {
    let line = "2025-01-15 14:29:55 [POSTINST] postinst triggered — action: configure";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::PostInst);
}

#[test]
fn parse_unknown_level_tag() {
    let line = "2025-01-15 14:31:30 [WEIRD] something unusual";
    let entry = parse(line).expect("must parse even with unknown level");
    assert_eq!(entry.level, LogLevel::Unknown);
    assert_eq!(entry.message, "something unusual");
}

#[test]
fn parse_no_bracket_returns_unknown() {
    let line = "2025-01-15 14:31:30 plain text without bracket";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.level, LogLevel::Unknown);
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
    // Lines shorter than 20 chars can't have a valid timestamp.
    assert!(parse("2025-01-15 14:30").is_none());
}

#[test]
fn parse_preserves_timestamp_exactly() {
    let line = "2026-03-31 23:59:59 [OK]    done";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.timestamp, "2026-03-31 23:59:59");
}

#[test]
fn parse_message_with_colon() {
    let line = "2025-06-01 10:00:00 [INFO]  repos_root: /home/user/projects";
    let entry = parse(line).expect("must parse");
    assert_eq!(entry.message, "repos_root: /home/user/projects");
}

#[test]
fn parse_message_trimmed_of_whitespace() {
    let line = "2025-06-01 10:00:00 [OK]      extra spaces before message   ";
    let entry = parse(line).expect("must parse");
    // The message should be trimmed at both ends.
    assert!(!entry.message.starts_with(' '));
    assert!(!entry.message.ends_with(' '));
}

#[test]
fn only_error_and_warn_are_issues() {
    let lines = vec![
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
