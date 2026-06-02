use super::sections;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::settings::ConfigSection;
use crate::lang::{lang_map::config as keys, text as tr};
use crate::messages::{ConfigMessage, Message};
use crate::ui::icons::Icon;
use crate::ui::tabs::config::ConfigTab;
use crate::ui::tabs::ssh_keys::SshKeysTab;
use crate::ui::templates::prelude as ui;
use iced::Element;
use iced::widget::column;

pub(super) fn config_sections<'a>(
    tab: &'a ConfigTab,
    ssh_keys_tab: &'a SshKeysTab,
    compact: bool,
) -> Element<'a, Message> {
    let sections = [
        ConfigSection::Ui,
        ConfigSection::UiConfig,
        ConfigSection::Apache,
        ConfigSection::Php,
        ConfigSection::Editor,
        ConfigSection::Advanced,
    ];

    ui::row_group(
        sections
            .iter()
            .map(|section| config_section_block(tab, ssh_keys_tab, section.clone(), compact))
            .collect(),
    )
}

fn config_section_block<'a>(
    tab: &'a ConfigTab,
    ssh_keys_tab: &'a SshKeysTab,
    section: ConfigSection,
    compact: bool,
) -> Element<'a, Message> {
    let expanded = tab.active_section.as_ref() == Some(&section);
    let can_save = expanded && tab.has_unsaved_changes() && !tab.saving;
    let actions = if expanded {
        if tab.has_unsaved_changes() {
            vec![ui::compact_action_button(
                if tab.saving {
                    tr(keys::SAVING)
                } else {
                    tr(keys::SAVE_CHANGES)
                },
                theme::color(theme_keys::TEAL),
                theme::color(theme_keys::TEAL_BG),
                theme::color(theme_keys::TEAL_HOVER),
                theme::color(theme_keys::TEAL_BORDER),
                if can_save {
                    Some(Message::Config(ConfigMessage::Save))
                } else {
                    None
                },
            )]
        } else {
            vec![]
        }
    } else {
        vec![ui::compact_action_button(
            "Edit",
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            Some(Message::Config(ConfigMessage::ToggleSection(
                section.clone(),
            ))),
        )]
    };

    let row = ui::summary_row(
        section_icon(&section),
        section_label(&section),
        section_summary(tab, &section),
        ui::BadgeTone::Info,
        actions,
        expanded,
        Some(Message::Config(ConfigMessage::ToggleSection(
            section.clone(),
        ))),
    );

    if !expanded {
        return row;
    }

    column![
        row,
        sections::section_body(tab, ssh_keys_tab, section, compact)
    ]
    .spacing(6)
    .into()
}

fn section_icon(section: &ConfigSection) -> Icon {
    match section {
        ConfigSection::Ui => Icon::Config,
        ConfigSection::UiConfig => Icon::Config,
        ConfigSection::Apache => Icon::Apache,
        ConfigSection::Php => Icon::Php,
        ConfigSection::Editor => Icon::Editor,
        ConfigSection::Advanced => Icon::Shield,
    }
}

fn section_label(section: &ConfigSection) -> &'static str {
    match section {
        ConfigSection::Ui => tr(keys::SECTION_UI),
        ConfigSection::UiConfig => tr(keys::SECTION_UI_CONFIG),
        ConfigSection::Apache => tr(keys::SECTION_APACHE),
        ConfigSection::Php => tr(keys::SECTION_PHP),
        ConfigSection::Editor => tr(keys::SECTION_EDITOR),
        ConfigSection::Advanced => tr(keys::SECTION_ADVANCED),
    }
}

fn section_summary(tab: &ConfigTab, section: &ConfigSection) -> String {
    match section {
        ConfigSection::Ui => format!(
            "Theme: {}, Language: {}",
            tab.settings.ui_theme, tab.settings.ui_language
        ),
        ConfigSection::UiConfig => "JSON override, layout and type".to_string(),
        ConfigSection::Apache => format!(
            "Auto reload {}, log {}",
            setting_state(tab.settings.apache_auto_reload),
            tab.settings.apache_log_level
        ),
        ConfigSection::Php => format!(
            "Default {}, display_errors {}",
            tab.settings.php_default_version,
            setting_state(tab.settings.php_display_errors)
        ),
        ConfigSection::Editor => format!(
            "Editor: {}, open: {}",
            tab.settings.editor_command, tab.settings.projects_open_command
        ),
        ConfigSection::Advanced => format!(
            "SSH {}, setup warnings {}",
            tab.settings.ssh_default_key_type,
            setting_state(tab.settings.ui_show_setup_log)
        ),
    }
}

fn setting_state(enabled: bool) -> &'static str {
    if enabled {
        tr(keys::ENABLED)
    } else {
        tr(keys::DISABLED)
    }
}
