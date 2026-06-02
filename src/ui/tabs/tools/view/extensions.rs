use super::shared::{search_box, section_header, small_action_btn, tool_item_row};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::tools::PhpExtension;
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::tools::ToolsTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, text};
use iced::{Element, Length, Padding};

pub(super) fn php_exts_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let active_ver: Option<String> = tab
        .php_releases
        .iter()
        .find(|r| r.is_active)
        .map(|r| r.version.clone());
    let ver_label = active_ver
        .as_deref()
        .unwrap_or(tr(keys::ACTIVE_VERSION_FALLBACK));

    let header = section_header(
        tr(keys::SECTION_PHP_EXTENSIONS),
        format!(
            "{} {} {}",
            tr(keys::PHP_EXTENSIONS_HELP_PREFIX),
            ver_label,
            tr(keys::PHP_EXTENSIONS_HELP_SUFFIX)
        ),
        tr(keys::SCAN),
        Some(Message::Tools(ToolsMessage::ScanPhpExts)),
    );

    let q = tab.tool_search.to_lowercase();
    let rows: Vec<Element<Message>> = tab
        .php_exts
        .iter()
        .filter(|e| q.is_empty() || e.name.contains(&q) || e.pkg_suffix.contains(&q))
        .map(|e| php_ext_row(e, &active_ver))
        .collect();
    container(
        column![
            header,
            Space::with_height(18),
            search_box(tr(keys::SEARCH_PLACEHOLDER), &tab.tool_search),
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(14),
            column(rows).spacing(8),
            Space::with_height(16),
            ui::info_banner(
                Icon::Info,
                text(tr(keys::PHP_EXTENSIONS_NOTE))
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .into(),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_BORDER),
            ),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn php_ext_row<'a>(ext: &'a PhpExtension, active_ver: &Option<String>) -> Element<'a, Message> {
    let (dot_color, status_text) = if ext.installed {
        (theme::color(theme_keys::GREEN), tr(keys::STATUS_INSTALLED))
    } else {
        (
            theme::color(theme_keys::TEXT_MUTED),
            tr(keys::STATUS_NOT_INSTALLED),
        )
    };
    let pkg = match active_ver {
        Some(ver) => format!("php{}-{}", ver, ext.name),
        None => ext.pkg_suffix.clone(),
    };
    let action: Element<Message> = if ext.installed {
        small_action_btn(
            tr(keys::REMOVE),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            Message::Tools(ToolsMessage::RemovePhpExt(pkg)),
        )
    } else {
        small_action_btn(
            tr(keys::INSTALL),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::InstallPhpExt(pkg)),
        )
    };
    tool_item_row(
        ext.name.as_str(),
        ext.pkg_suffix.as_str(),
        status_text,
        dot_color,
        action,
    )
}
