// src/tabs/apache_touch/mod.rs — state, data types, public API
#![allow(dead_code, unused)]

pub mod backend;
pub mod view;

use iced::Element;
use crate::Message;

#[derive(Debug, Clone)]
pub enum LogKind { Info, Success, Warning, Error, Cmd }

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub kind:    LogKind,
    pub message: String,
}

impl LogEntry {
    pub fn info(msg: impl Into<String>) -> Self { Self { kind: LogKind::Info,    message: msg.into() } }
    pub fn ok(msg:   impl Into<String>) -> Self { Self { kind: LogKind::Success, message: msg.into() } }
    pub fn warn(msg: impl Into<String>) -> Self { Self { kind: LogKind::Warning, message: msg.into() } }
    pub fn err(msg:  impl Into<String>) -> Self { Self { kind: LogKind::Error,   message: msg.into() } }
    pub fn cmd(msg:  impl Into<String>) -> Self { Self { kind: LogKind::Cmd,     message: msg.into() } }
}

pub struct ApacheTouchTab {
    pub project_name:   String,
    pub auth_json_path: String,
    pub base_dir:       String,
    pub apache_conf:    String,
    pub log:            Vec<LogEntry>,
    pub running:        bool,
    pub finished_ok:    Option<bool>,
}

impl ApacheTouchTab {
    pub fn new() -> Self {
        Self {
            project_name:   String::new(),
            auth_json_path: String::new(),
            base_dir:       "/var/www".into(),
            apache_conf:    "/etc/apache2/sites-available/projects.conf".into(),
            log:            Vec::new(),
            running:        false,
            finished_ok:    None,
        }
    }

    pub fn push_log(&mut self, entry: LogEntry) {
        self.log.push(entry);
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
        self.finished_ok = None;
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
