pub const LOG_PATH: &str = crate::core::paths::SETUP_LOG;

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Step,
    Ok,
    Info,
    Warn,
    Error,
    Cmd,
    Out,
    PostInst,
    Unknown,
}

impl LogLevel {
    pub fn as_tag(&self) -> &'static str {
        match self {
            LogLevel::Step => "STEP",
            LogLevel::Ok => "OK",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Cmd => "CMD",
            LogLevel::Out => "OUT",
            LogLevel::PostInst => "POSTINST",
            LogLevel::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetupLogEntry {
    #[allow(dead_code)]
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

impl SetupLogEntry {
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.len() < 20 {
            return None;
        }
        let timestamp = line[..19].to_string();
        let rest = line[20..].trim();
        let (level, message) = if let Some(bracket_end) = rest.find(']') {
            let level_str = rest[1..bracket_end].trim();
            let msg = rest[bracket_end + 1..].trim().to_string();
            let level = match level_str {
                "STEP" => LogLevel::Step,
                "OK" => LogLevel::Ok,
                "INFO" => LogLevel::Info,
                "WARN" => LogLevel::Warn,
                "ERROR" => LogLevel::Error,
                "CMD" => LogLevel::Cmd,
                "OUT" => LogLevel::Out,
                "POSTINST" => LogLevel::PostInst,
                _ => LogLevel::Unknown,
            };
            (level, msg)
        } else {
            (LogLevel::Unknown, rest.to_string())
        };
        Some(SetupLogEntry {
            timestamp,
            level,
            message,
        })
    }
}

pub fn format_setup_log_line(timestamp: &str, level: LogLevel, message: &str) -> String {
    format!("{} [{:<5}] {}", timestamp, level.as_tag(), message)
}

pub fn append_setup_log(level: LogLevel, message: &str) {
    if crate::core::dry_run::active() {
        crate::core::dry_run::log(&format!("setup_log [{}] {}", level.as_tag(), message));
        return;
    }

    if let Some(parent) = std::path::Path::new(LOG_PATH).parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!("DevPanel setup log: failed to create log dir: {e}");
        return;
    }

    let line = format_setup_log_line(&current_timestamp_utc(), level, message);
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
    {
        Ok(mut file) => {
            let _ = writeln!(file, "{line}");
        }
        Err(e) => {
            eprintln!("DevPanel setup log: failed to write {LOG_PATH}: {e}");
        }
    }
}

fn current_timestamp_utc() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_timestamp_utc(seconds)
}

fn format_unix_timestamp_utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[allow(dead_code)]
pub fn setup_has_run() -> bool {
    std::path::Path::new(LOG_PATH).exists()
}

pub fn read_setup_log() -> Vec<SetupLogEntry> {
    let content = match std::fs::read_to_string(LOG_PATH) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content.lines().filter_map(SetupLogEntry::parse).collect()
}

pub async fn read_setup_log_async() -> Vec<SetupLogEntry> {
    match tokio::fs::read_to_string(LOG_PATH).await {
        Ok(content) => content.lines().filter_map(SetupLogEntry::parse).collect(),
        Err(_) => Vec::new(),
    }
}

pub fn read_setup_issues() -> Vec<SetupLogEntry> {
    read_setup_log()
        .into_iter()
        .filter(|e| matches!(e.level, LogLevel::Error | LogLevel::Warn))
        .collect()
}
