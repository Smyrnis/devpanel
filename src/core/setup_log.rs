pub const LOG_PATH: &str = crate::core::paths::SETUP_LOG;

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SetupLogEntry {
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

#[allow(dead_code)]
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
