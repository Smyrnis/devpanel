use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

const UI_FALLBACK: &str = include_str!("../../share/ui/config.json");
const PHP_FALLBACK: &str = include_str!("../../share/versions/php.json");

static UI_CONFIG: OnceLock<RwLock<Value>> = OnceLock::new();
static PHP_CONFIG: OnceLock<Value> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct PhpVersionSpec {
    pub version: String,
    pub binaries: Vec<String>,
    pub apt_package: String,
    pub fpm_package: String,
    pub fpm_service: String,
    pub fpm_conf: String,
    pub fpm_socket: String,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowMetrics {
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TextMetrics {
    pub title: u16,
    pub modal_title: u16,
    pub dialog_title: u16,
    pub section_title: u16,
    pub body: u16,
    pub caption: u16,
    pub tiny: u16,
    pub badge: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct IconMetrics {
    pub sidebar_logo: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ControlMetrics {
    pub button_height: f32,
    pub summary_row_height: f32,
    pub detail_label_width: f32,
    pub checkbox_size: u16,
    pub large_checkbox_size: u16,
    pub modal_log_height: f32,
    pub form_dropdown_width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PanelMetrics {
    pub notification_width: f32,
    pub sudo_dialog_width: f32,
    pub installer_log_height: f32,
    pub ssh_keys_list_height: f32,
    pub tools_list_height: f32,
    pub tools_log_height: f32,
    pub tools_compact_log_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiConfigField {
    WindowWidth,
    WindowHeight,
    WindowMinWidth,
    WindowMinHeight,
    SidebarCollapseWidth,
    InstallerContentWidth,
    TextTitle,
    TextModalTitle,
    TextDialogTitle,
    TextSectionTitle,
    TextBody,
    TextCaption,
    TextTiny,
    TextBadge,
    IconSidebarLogo,
    ControlButtonHeight,
    ControlSummaryRowHeight,
    ControlDetailLabelWidth,
    ControlCheckboxSize,
    ControlLargeCheckboxSize,
    ControlModalLogHeight,
    ControlFormDropdownWidth,
    PanelNotificationWidth,
    PanelSudoDialogWidth,
    PanelInstallerLogHeight,
    PanelSshKeysListHeight,
    PanelToolsListHeight,
    PanelToolsLogHeight,
    PanelToolsCompactLogHeight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConfigDraft {
    pub window_width: String,
    pub window_height: String,
    pub window_min_width: String,
    pub window_min_height: String,
    pub sidebar_collapse_width: String,
    pub installer_content_width: String,
    pub text_title: String,
    pub text_modal_title: String,
    pub text_dialog_title: String,
    pub text_section_title: String,
    pub text_body: String,
    pub text_caption: String,
    pub text_tiny: String,
    pub text_badge: String,
    pub icon_sidebar_logo: String,
    pub control_button_height: String,
    pub control_summary_row_height: String,
    pub control_detail_label_width: String,
    pub control_checkbox_size: String,
    pub control_large_checkbox_size: String,
    pub control_modal_log_height: String,
    pub control_form_dropdown_width: String,
    pub panel_notification_width: String,
    pub panel_sudo_dialog_width: String,
    pub panel_installer_log_height: String,
    pub panel_ssh_keys_list_height: String,
    pub panel_tools_list_height: String,
    pub panel_tools_log_height: String,
    pub panel_tools_compact_log_height: String,
}

impl UiConfigDraft {
    pub fn current() -> Self {
        let window = window_metrics();
        let text = text_metrics();
        let icons = icon_metrics();
        let controls = control_metrics();
        let panels = panel_metrics();

        Self {
            window_width: format_number(window.width),
            window_height: format_number(window.height),
            window_min_width: format_number(window.min_width),
            window_min_height: format_number(window.min_height),
            sidebar_collapse_width: format_number(sidebar_collapse_width()),
            installer_content_width: format_number(installer_content_width()),
            text_title: text.title.to_string(),
            text_modal_title: text.modal_title.to_string(),
            text_dialog_title: text.dialog_title.to_string(),
            text_section_title: text.section_title.to_string(),
            text_body: text.body.to_string(),
            text_caption: text.caption.to_string(),
            text_tiny: text.tiny.to_string(),
            text_badge: text.badge.to_string(),
            icon_sidebar_logo: format_number(icons.sidebar_logo),
            control_button_height: format_number(controls.button_height),
            control_summary_row_height: format_number(controls.summary_row_height),
            control_detail_label_width: format_number(controls.detail_label_width),
            control_checkbox_size: controls.checkbox_size.to_string(),
            control_large_checkbox_size: controls.large_checkbox_size.to_string(),
            control_modal_log_height: format_number(controls.modal_log_height),
            control_form_dropdown_width: format_number(controls.form_dropdown_width),
            panel_notification_width: format_number(panels.notification_width),
            panel_sudo_dialog_width: format_number(panels.sudo_dialog_width),
            panel_installer_log_height: format_number(panels.installer_log_height),
            panel_ssh_keys_list_height: format_number(panels.ssh_keys_list_height),
            panel_tools_list_height: format_number(panels.tools_list_height),
            panel_tools_log_height: format_number(panels.tools_log_height),
            panel_tools_compact_log_height: format_number(panels.tools_compact_log_height),
        }
    }

    pub fn set_field(&mut self, field: UiConfigField, value: String) {
        match field {
            UiConfigField::WindowWidth => self.window_width = value,
            UiConfigField::WindowHeight => self.window_height = value,
            UiConfigField::WindowMinWidth => self.window_min_width = value,
            UiConfigField::WindowMinHeight => self.window_min_height = value,
            UiConfigField::SidebarCollapseWidth => self.sidebar_collapse_width = value,
            UiConfigField::InstallerContentWidth => self.installer_content_width = value,
            UiConfigField::TextTitle => self.text_title = value,
            UiConfigField::TextModalTitle => self.text_modal_title = value,
            UiConfigField::TextDialogTitle => self.text_dialog_title = value,
            UiConfigField::TextSectionTitle => self.text_section_title = value,
            UiConfigField::TextBody => self.text_body = value,
            UiConfigField::TextCaption => self.text_caption = value,
            UiConfigField::TextTiny => self.text_tiny = value,
            UiConfigField::TextBadge => self.text_badge = value,
            UiConfigField::IconSidebarLogo => self.icon_sidebar_logo = value,
            UiConfigField::ControlButtonHeight => self.control_button_height = value,
            UiConfigField::ControlSummaryRowHeight => self.control_summary_row_height = value,
            UiConfigField::ControlDetailLabelWidth => self.control_detail_label_width = value,
            UiConfigField::ControlCheckboxSize => self.control_checkbox_size = value,
            UiConfigField::ControlLargeCheckboxSize => self.control_large_checkbox_size = value,
            UiConfigField::ControlModalLogHeight => self.control_modal_log_height = value,
            UiConfigField::ControlFormDropdownWidth => self.control_form_dropdown_width = value,
            UiConfigField::PanelNotificationWidth => self.panel_notification_width = value,
            UiConfigField::PanelSudoDialogWidth => self.panel_sudo_dialog_width = value,
            UiConfigField::PanelInstallerLogHeight => self.panel_installer_log_height = value,
            UiConfigField::PanelSshKeysListHeight => self.panel_ssh_keys_list_height = value,
            UiConfigField::PanelToolsListHeight => self.panel_tools_list_height = value,
            UiConfigField::PanelToolsLogHeight => self.panel_tools_log_height = value,
            UiConfigField::PanelToolsCompactLogHeight => {
                self.panel_tools_compact_log_height = value
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.values().map(|_| ())
    }

    fn values(&self) -> Result<UiConfigValues, String> {
        Ok(UiConfigValues {
            window_width: parse_f32("window.width", &self.window_width, 320.0)?,
            window_height: parse_f32("window.height", &self.window_height, 320.0)?,
            window_min_width: parse_f32("window.min_width", &self.window_min_width, 320.0)?,
            window_min_height: parse_f32("window.min_height", &self.window_min_height, 320.0)?,
            sidebar_collapse_width: parse_f32(
                "layout.sidebar_collapse_width",
                &self.sidebar_collapse_width,
                320.0,
            )?,
            installer_content_width: parse_f32(
                "installer.content_width",
                &self.installer_content_width,
                320.0,
            )?,
            text_title: parse_u16("text.title", &self.text_title, 8, 72)?,
            text_modal_title: parse_u16("text.modal_title", &self.text_modal_title, 8, 72)?,
            text_dialog_title: parse_u16("text.dialog_title", &self.text_dialog_title, 8, 72)?,
            text_section_title: parse_u16("text.section_title", &self.text_section_title, 8, 72)?,
            text_body: parse_u16("text.body", &self.text_body, 8, 72)?,
            text_caption: parse_u16("text.caption", &self.text_caption, 8, 72)?,
            text_tiny: parse_u16("text.tiny", &self.text_tiny, 8, 72)?,
            text_badge: parse_u16("text.badge", &self.text_badge, 8, 72)?,
            icon_sidebar_logo: parse_f32("icons.sidebar_logo", &self.icon_sidebar_logo, 8.0)?,
            control_button_height: parse_f32(
                "controls.button_height",
                &self.control_button_height,
                20.0,
            )?,
            control_summary_row_height: parse_f32(
                "controls.summary_row_height",
                &self.control_summary_row_height,
                20.0,
            )?,
            control_detail_label_width: parse_f32(
                "controls.detail_label_width",
                &self.control_detail_label_width,
                40.0,
            )?,
            control_checkbox_size: parse_u16(
                "controls.checkbox_size",
                &self.control_checkbox_size,
                8,
                40,
            )?,
            control_large_checkbox_size: parse_u16(
                "controls.large_checkbox_size",
                &self.control_large_checkbox_size,
                8,
                40,
            )?,
            control_modal_log_height: parse_f32(
                "controls.modal_log_height",
                &self.control_modal_log_height,
                80.0,
            )?,
            control_form_dropdown_width: parse_f32(
                "controls.form_dropdown_width",
                &self.control_form_dropdown_width,
                80.0,
            )?,
            panel_notification_width: parse_f32(
                "panels.notification_width",
                &self.panel_notification_width,
                180.0,
            )?,
            panel_sudo_dialog_width: parse_f32(
                "panels.sudo_dialog_width",
                &self.panel_sudo_dialog_width,
                280.0,
            )?,
            panel_installer_log_height: parse_f32(
                "panels.installer_log_height",
                &self.panel_installer_log_height,
                80.0,
            )?,
            panel_ssh_keys_list_height: parse_f32(
                "panels.ssh_keys_list_height",
                &self.panel_ssh_keys_list_height,
                120.0,
            )?,
            panel_tools_list_height: parse_f32(
                "panels.tools_list_height",
                &self.panel_tools_list_height,
                120.0,
            )?,
            panel_tools_log_height: parse_f32(
                "panels.tools_log_height",
                &self.panel_tools_log_height,
                80.0,
            )?,
            panel_tools_compact_log_height: parse_f32(
                "panels.tools_compact_log_height",
                &self.panel_tools_compact_log_height,
                80.0,
            )?,
        })
    }
}

pub fn window_metrics() -> WindowMetrics {
    with_ui_config(|root| WindowMetrics {
        width: number(root, &["window", "width"], 1040.0),
        height: number(root, &["window", "height"], 660.0),
        min_width: number(root, &["window", "min_width"], 680.0),
        min_height: number(root, &["window", "min_height"], 520.0),
    })
}

pub fn sidebar_collapse_width() -> f32 {
    with_ui_config(|root| number(root, &["layout", "sidebar_collapse_width"], 786.0))
}

pub fn installer_content_width() -> f32 {
    with_ui_config(|root| number(root, &["installer", "content_width"], 620.0))
}

pub fn text_metrics() -> TextMetrics {
    with_ui_config(|root| TextMetrics {
        title: integer(root, &["text", "title"], 22),
        modal_title: integer(root, &["text", "modal_title"], 18),
        dialog_title: integer(root, &["text", "dialog_title"], 16),
        section_title: integer(root, &["text", "section_title"], 15),
        body: integer(root, &["text", "body"], 13),
        caption: integer(root, &["text", "caption"], 11),
        tiny: integer(root, &["text", "tiny"], 10),
        badge: integer(root, &["text", "badge"], 9),
    })
}

pub fn icon_metrics() -> IconMetrics {
    with_ui_config(|root| IconMetrics {
        sidebar_logo: number(root, &["icons", "sidebar_logo"], 19.0),
    })
}

pub fn control_metrics() -> ControlMetrics {
    with_ui_config(|root| ControlMetrics {
        button_height: number(root, &["controls", "button_height"], 38.0),
        summary_row_height: number(root, &["controls", "summary_row_height"], 34.0),
        detail_label_width: number(root, &["controls", "detail_label_width"], 130.0),
        checkbox_size: integer(root, &["controls", "checkbox_size"], 16),
        large_checkbox_size: integer(root, &["controls", "large_checkbox_size"], 18),
        modal_log_height: number(root, &["controls", "modal_log_height"], 240.0),
        form_dropdown_width: number(root, &["controls", "form_dropdown_width"], 220.0),
    })
}

pub fn panel_metrics() -> PanelMetrics {
    with_ui_config(|root| PanelMetrics {
        notification_width: number(root, &["panels", "notification_width"], 340.0),
        sudo_dialog_width: number(root, &["panels", "sudo_dialog_width"], 420.0),
        installer_log_height: number(root, &["panels", "installer_log_height"], 110.0),
        ssh_keys_list_height: number(root, &["panels", "ssh_keys_list_height"], 380.0),
        tools_list_height: number(root, &["panels", "tools_list_height"], 420.0),
        tools_log_height: number(root, &["panels", "tools_log_height"], 180.0),
        tools_compact_log_height: number(root, &["panels", "tools_compact_log_height"], 150.0),
    })
}

pub fn user_ui_config_path() -> PathBuf {
    user_config_dir().join("ui").join("config.json")
}

pub fn save_user_ui_config(draft: &UiConfigDraft) -> Result<PathBuf, String> {
    let values = draft.values()?;
    let value = values.to_json();
    let path = user_ui_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("Failed to serialize UI config: {error}"))?
        + "\n";
    std::fs::write(&path, contents)
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))?;
    replace_ui_config(value);
    Ok(path)
}

pub fn latest_php_version() -> String {
    string(php_config(), &["latest"], "8.5")
}

pub fn php_versions() -> Vec<PhpVersionSpec> {
    let Some(items) = value(php_config(), &["available"]).and_then(Value::as_array) else {
        return fallback_php_versions();
    };

    let versions: Vec<_> = items
        .iter()
        .filter_map(|item| {
            let version = item.get("version")?.as_str()?.to_string();
            let apt_package = item
                .get("apt_package")
                .and_then(Value::as_str)
                .unwrap_or(version.as_str())
                .to_string();
            let binaries = item
                .get("binaries")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .filter(|items| !items.is_empty())
                .unwrap_or_else(|| vec![format!("php{version}")]);

            Some(PhpVersionSpec {
                version: version.clone(),
                binaries,
                apt_package,
                fpm_package: item
                    .get("fpm_package")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("php{version}-fpm")),
                fpm_service: item
                    .get("fpm_service")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("php{version}-fpm")),
                fpm_conf: item
                    .get("fpm_conf")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("php{version}-fpm")),
                fpm_socket: item
                    .get("fpm_socket")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("/run/php/php{version}-fpm.sock")),
            })
        })
        .collect();

    if versions.is_empty() {
        fallback_php_versions()
    } else {
        versions
    }
}

pub fn php_version_numbers() -> Vec<String> {
    php_versions()
        .into_iter()
        .map(|spec| spec.version)
        .collect()
}

struct UiConfigValues {
    window_width: f32,
    window_height: f32,
    window_min_width: f32,
    window_min_height: f32,
    sidebar_collapse_width: f32,
    installer_content_width: f32,
    text_title: u16,
    text_modal_title: u16,
    text_dialog_title: u16,
    text_section_title: u16,
    text_body: u16,
    text_caption: u16,
    text_tiny: u16,
    text_badge: u16,
    icon_sidebar_logo: f32,
    control_button_height: f32,
    control_summary_row_height: f32,
    control_detail_label_width: f32,
    control_checkbox_size: u16,
    control_large_checkbox_size: u16,
    control_modal_log_height: f32,
    control_form_dropdown_width: f32,
    panel_notification_width: f32,
    panel_sudo_dialog_width: f32,
    panel_installer_log_height: f32,
    panel_ssh_keys_list_height: f32,
    panel_tools_list_height: f32,
    panel_tools_log_height: f32,
    panel_tools_compact_log_height: f32,
}

impl UiConfigValues {
    fn to_json(&self) -> Value {
        json!({
            "window": {
                "width": self.window_width,
                "height": self.window_height,
                "min_width": self.window_min_width,
                "min_height": self.window_min_height
            },
            "layout": {
                "sidebar_collapse_width": self.sidebar_collapse_width
            },
            "installer": {
                "content_width": self.installer_content_width
            },
            "text": {
                "title": self.text_title,
                "modal_title": self.text_modal_title,
                "dialog_title": self.text_dialog_title,
                "section_title": self.text_section_title,
                "body": self.text_body,
                "caption": self.text_caption,
                "tiny": self.text_tiny,
                "badge": self.text_badge
            },
            "icons": {
                "sidebar_logo": self.icon_sidebar_logo
            },
            "controls": {
                "button_height": self.control_button_height,
                "summary_row_height": self.control_summary_row_height,
                "detail_label_width": self.control_detail_label_width,
                "checkbox_size": self.control_checkbox_size,
                "large_checkbox_size": self.control_large_checkbox_size,
                "modal_log_height": self.control_modal_log_height,
                "form_dropdown_width": self.control_form_dropdown_width
            },
            "panels": {
                "notification_width": self.panel_notification_width,
                "sudo_dialog_width": self.panel_sudo_dialog_width,
                "installer_log_height": self.panel_installer_log_height,
                "ssh_keys_list_height": self.panel_ssh_keys_list_height,
                "tools_list_height": self.panel_tools_list_height,
                "tools_log_height": self.panel_tools_log_height,
                "tools_compact_log_height": self.panel_tools_compact_log_height
            }
        })
    }
}

fn with_ui_config<T>(f: impl FnOnce(&Value) -> T) -> T {
    let lock = UI_CONFIG.get_or_init(|| RwLock::new(load_ui_config()));
    let guard = match lock.read() {
        Ok(guard) => guard,
        Err(error) => error.into_inner(),
    };
    f(&guard)
}

fn replace_ui_config(value: Value) {
    let lock = UI_CONFIG.get_or_init(|| RwLock::new(load_ui_config()));
    let mut guard = match lock.write() {
        Ok(guard) => guard,
        Err(error) => error.into_inner(),
    };
    *guard = value;
}

fn load_ui_config() -> Value {
    load_config_from_paths(ui_config_paths(), UI_FALLBACK)
}

fn php_config() -> &'static Value {
    PHP_CONFIG.get_or_init(|| {
        load_config(
            "DEVPANEL_PHP_VERSIONS",
            &["share", "versions", "php.json"],
            "/usr/share/devpanel/versions/php.json",
            PHP_FALLBACK,
        )
    })
}

fn load_config(env_key: &str, local_parts: &[&str], installed_path: &str, fallback: &str) -> Value {
    load_config_from_paths(config_paths(env_key, local_parts, installed_path), fallback)
}

fn load_config_from_paths(paths: Vec<PathBuf>, fallback: &str) -> Value {
    for path in paths {
        let Ok(contents) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Ok(value) = serde_json::from_str(&contents) {
            return value;
        }
    }
    serde_json::from_str(fallback).unwrap_or(Value::Null)
}

fn ui_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(path) = std::env::var("DEVPANEL_UI_CONFIG") {
        paths.push(PathBuf::from(path));
    }
    paths.push(user_ui_config_path());
    if let Ok(root) = std::env::current_dir() {
        paths.push(root.join("share").join("ui").join("config.json"));
    }
    paths.push(PathBuf::from("/usr/share/devpanel/ui/config.json"));
    paths
}

fn config_paths(env_key: &str, local_parts: &[&str], installed_path: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(path) = std::env::var(env_key) {
        paths.push(PathBuf::from(path));
    }
    if let Ok(root) = std::env::current_dir() {
        let mut path = root;
        for part in local_parts {
            path = path.join(part);
        }
        paths.push(path);
    }
    paths.push(PathBuf::from(installed_path));
    paths
}

fn user_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config").join("devpanel")
}

fn value<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn number(root: &Value, path: &[&str], fallback: f32) -> f32 {
    value(root, path)
        .and_then(Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(fallback)
}

fn integer(root: &Value, path: &[&str], fallback: u16) -> u16 {
    value(root, path)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(fallback)
}

fn string(root: &Value, path: &[&str], fallback: &str) -> String {
    value(root, path)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn parse_f32(label: &str, input: &str, min: f32) -> Result<f32, String> {
    let value = input
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("{label} must be a number"))?;
    if !value.is_finite() || value < min {
        return Err(format!("{label} must be at least {min}"));
    }
    Ok(value)
}

fn parse_u16(label: &str, input: &str, min: u16, max: u16) -> Result<u16, String> {
    let value = input
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("{label} must be a whole number"))?;
    if value < min || value > max {
        return Err(format!("{label} must be between {min} and {max}"));
    }
    Ok(value)
}

fn format_number(value: f32) -> String {
    if (value.fract()).abs() < f32::EPSILON {
        format!("{}", value as i32)
    } else {
        value.to_string()
    }
}

fn fallback_php_versions() -> Vec<PhpVersionSpec> {
    [
        "5.6", "7.0", "7.1", "7.2", "7.3", "7.4", "8.0", "8.1", "8.2", "8.3", "8.4", "8.5",
    ]
    .into_iter()
    .map(|version| PhpVersionSpec {
        version: version.to_string(),
        binaries: if version == "5.6" {
            vec!["php5.6".to_string(), "php5".to_string()]
        } else {
            vec![format!("php{version}")]
        },
        apt_package: format!("php{version}"),
        fpm_package: format!("php{version}-fpm"),
        fpm_service: format!("php{version}-fpm"),
        fpm_conf: format!("php{version}-fpm"),
        fpm_socket: format!("/run/php/php{version}-fpm.sock"),
    })
    .collect()
}
