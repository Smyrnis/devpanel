pub mod lang_map;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

static LANGUAGE: OnceLock<RwLock<HashMap<String, &'static str>>> = OnceLock::new();

pub fn text(key: &'static str) -> &'static str {
    let language = LANGUAGE.get_or_init(|| RwLock::new(load_configured_language()));
    let Ok(language) = language.read() else {
        return key;
    };
    language.get(key).copied().unwrap_or(key)
}

pub fn set_language(code: &str) {
    let language = LANGUAGE.get_or_init(|| RwLock::new(load_configured_language()));
    if let Some(next) = load_language(code)
        && let Ok(mut current) = language.write()
    {
        *current = next;
    }
}

pub fn available_languages() -> Vec<String> {
    available_json_codes("languages", "/usr/share/devpanel/languages")
}

fn load_configured_language() -> HashMap<String, &'static str> {
    let code = configured_language_code();
    load_language(&code)
        .unwrap_or_else(|| parse_language(include_str!("../../share/languages/en.json")))
}

fn configured_language_code() -> String {
    if let Ok(code) = std::env::var("DEVPANEL_LANG") {
        return code;
    }
    if let Ok(db) = crate::core::db::DevPanelDb::open() {
        return crate::core::db::UserSettings::load(&db).ui_language;
    }
    "en".to_string()
}

fn load_language(code: &str) -> Option<HashMap<String, &'static str>> {
    let file_name = format!("{code}.json");
    for path in language_paths(&file_name) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            continue;
        };
        return Some(parse_language(&contents));
    }
    None
}

fn language_paths(file_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(root) = std::env::current_dir() {
        paths.push(root.join("share").join("languages").join(file_name));
    }
    paths.push(PathBuf::from("/usr/share/devpanel/languages").join(file_name));
    paths
}

fn parse_language(contents: &str) -> HashMap<String, &'static str> {
    serde_json::from_str::<HashMap<String, String>>(contents)
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| (key, leak_text(&value)))
        .collect()
}

fn available_json_codes(local_folder: &str, installed_folder: &str) -> Vec<String> {
    let mut codes = Vec::new();
    let mut folders = Vec::new();
    if let Ok(root) = std::env::current_dir() {
        folders.push(root.join("share").join(local_folder));
    }
    folders.push(PathBuf::from(installed_folder));

    for folder in folders {
        let Ok(entries) = std::fs::read_dir(folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|v| v.to_str())
                && !codes.iter().any(|code| code == stem)
            {
                codes.push(stem.to_string());
            }
        }
    }
    if codes.is_empty() {
        codes.push("en".to_string());
    }
    codes.sort();
    codes
}

fn leak_text(value: &str) -> &'static str {
    Box::leak(value.to_string().into_boxed_str())
}
