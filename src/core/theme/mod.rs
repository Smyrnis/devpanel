pub mod theme_map;

use iced::Color;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

static THEME: OnceLock<RwLock<HashMap<String, Color>>> = OnceLock::new();

pub fn color(key: &'static str) -> Color {
    let theme = THEME.get_or_init(|| RwLock::new(load_configured_theme()));
    let Ok(theme) = theme.read() else {
        return Color::TRANSPARENT;
    };
    theme
        .get(key)
        .copied()
        .or_else(|| parse_theme(include_str!("../../../share/themes/dark.json")).remove(key))
        .unwrap_or(Color::TRANSPARENT)
}

pub fn set_theme(code: &str) {
    let theme = THEME.get_or_init(|| RwLock::new(load_configured_theme()));
    if let Some(next) = load_theme(code)
        && let Ok(mut current) = theme.write()
    {
        *current = next;
    }
}

pub fn available_themes() -> Vec<String> {
    available_json_codes("themes", "/usr/share/devpanel/themes", "dark")
}

fn load_configured_theme() -> HashMap<String, Color> {
    let code = configured_theme_code();
    load_theme(&code)
        .unwrap_or_else(|| parse_theme(include_str!("../../../share/themes/dark.json")))
}

fn configured_theme_code() -> String {
    if let Ok(code) = std::env::var("DEVPANEL_THEME") {
        return code;
    }
    if let Ok(db) = crate::core::db::DevPanelDb::open() {
        return crate::core::db::UserSettings::load(&db).ui_theme;
    }
    "dark".to_string()
}

fn load_theme(code: &str) -> Option<HashMap<String, Color>> {
    let file_name = format!("{code}.json");
    for path in theme_paths(&file_name) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            continue;
        };
        return Some(parse_theme(&contents));
    }
    None
}

fn theme_paths(file_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(root) = std::env::current_dir() {
        paths.push(root.join("share").join("themes").join(file_name));
    }
    paths.push(PathBuf::from("/usr/share/devpanel/themes").join(file_name));
    paths
}

fn parse_theme(contents: &str) -> HashMap<String, Color> {
    let Ok(values) = serde_json::from_str::<HashMap<String, String>>(contents) else {
        return HashMap::new();
    };
    values
        .into_iter()
        .filter_map(|(key, value)| parse_hex_color(&value).map(|color| (key, color)))
        .collect()
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (rgb, alpha) = match hex.len() {
        6 => (hex, 255),
        8 => (&hex[..6], u8::from_str_radix(&hex[6..], 16).ok()?),
        _ => return None,
    };
    let r = u8::from_str_radix(&rgb[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb[4..6], 16).ok()?;
    Some(Color {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        a: f32::from(alpha) / 255.0,
    })
}

fn available_json_codes(local_folder: &str, installed_folder: &str, fallback: &str) -> Vec<String> {
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
        codes.push(fallback.to_string());
    }
    codes.sort();
    codes
}
