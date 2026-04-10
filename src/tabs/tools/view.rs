use super::{ApacheModule, PhpExtension, PhpRelease, PhpStatus, ToolSection, ToolsTab};
use crate::core::theme::*;
use crate::messages::{Message, ToolsMessage};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

const BLUE_BG: Color = Color { r: 0.050, g: 0.090, b: 0.180, a: 1.0 };
const BLUE_BORDER: Color = Color { r: 0.080, g: 0.140, b: 0.260, a: 1.0 };
const BLUE_HOVER: Color = Color { r: 0.070, g: 0.120, b: 0.230, a: 1.0 };
const GREEN_BG: Color = Color { r: 0.050, g: 0.160, b: 0.090, a: 1.0 };
const GREEN_HOVER: Color = Color { r: 0.060, g: 0.185, b: 0.100, a: 1.0 };
const RED_BG: Color = Color { r: 0.200, g: 0.060, b: 0.055, a: 1.0 };
const RED_HOVER: Color = Color { r: 0.230, g: 0.070, b: 0.063, a: 1.0 };
const YELLOW_BG: Color = Color { r: 0.190, g: 0.160, b: 0.040, a: 1.0 };
const YELLOW_BORDER: Color = Color { r: 0.240, g: 0.200, b: 0.050, a: 1.0 };
const TEAL_BG: Color = Color { r: 0.040, g: 0.160, b: 0.150, a: 1.0 };
const TEAL_BORDER: Color = Color { r: 0.060, g: 0.210, b: 0.200, a: 1.0 };
const TEAL_HOVER: Color = Color { r: 0.050, g: 0.185, b: 0.175, a: 1.0 };
const PURPLE_BG: Color = Color { r: 0.140, g: 0.060, b: 0.180, a: 1.0 };
const PURPLE_BORDER: Color = Color { r: 0.180, g: 0.080, b: 0.230, a: 1.0 };
const PURPLE_HOVER: Color = Color { r: 0.160, g: 0.070, b: 0.205, a: 1.0 };

pub fn render(tab: &ToolsTab) -> Element<'_, Message> {
    scrollable(
        column![
            column![
                text("Tools").size(22).color(TEXT_PRIMARY),
                Space::with_height(4),
                text("Manage PHP, Apache modules, extensions and database").size(13).color(TEXT_MUTED),
            ].spacing(0),
            Space::with_height(18),
            section_tabs(tab),
            Space::with_height(16),
            match tab.active_section {
                ToolSection::Php => php_panel(tab),
                ToolSection::ApacheMods => apache_mods_panel(tab),
                ToolSection::PhpExts => php_exts_panel(tab),
                ToolSection::Database => db_panel(tab),
            },
            Space::with_height(16),
            log_panel(tab),
            if tab.last_php_error.is_some() { Space::with_height(16) } else { Space::with_height(0) },
            if tab.last_php_error.is_some() { error_suggestion_panel(tab) } else { Space::with_height(0).into() },
            Space::with_height(22),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn section_tabs(tab: &ToolsTab) -> Element<'_, Message> {
    let sections = [
        (ToolSection::Php, "PHP Versions"),
        (ToolSection::ApacheMods, "Apache Modules"),
        (ToolSection::PhpExts, "PHP Extensions"),
        (ToolSection::Database, "Database CLI"),
    ];
    let tabs: Vec<Element<Message>> = sections.iter().map(|(sec, label)| {
        let active = *sec == tab.active_section;
        let (color, bg, bg_hover) = if active { (TEAL, TEAL_BG, TEAL_HOVER) } else { (TEXT_MUTED, BG_SURFACE, BG_HOVER) };
        let msg = match sec {
            ToolSection::Php => Message::Tools(ToolsMessage::SetSection(ToolSection::Php)),
            ToolSection::ApacheMods => Message::Tools(ToolsMessage::SetSection(ToolSection::ApacheMods)),
            ToolSection::PhpExts => Message::Tools(ToolsMessage::SetSection(ToolSection::PhpExts)),
            ToolSection::Database => Message::Tools(ToolsMessage::SetSection(ToolSection::Database)),
        };
        button(text(*label).size(12).color(color)).on_press(msg)
            .padding(Padding::from([7, 16]))
            .style(move |_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                    iced::widget::button::Style { background: Some(bg_hover.into()), text_color: color,
                        border: Border { color: if active { TEAL_BORDER } else { BORDER_SUBTLE }, width: 1.0, radius: 8.0.into() },
                        ..Default::default() },
                _ => iced::widget::button::Style { background: Some(bg.into()), text_color: color,
                    border: Border { color: if active { TEAL_BORDER } else { BORDER_SUBTLE }, width: 1.0, radius: 8.0.into() },
                    ..Default::default() },
            }).into()
    }).collect();
    row(tabs).spacing(8).into()
}

fn php_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_lbl = if tab.scanning { "Scanning…" } else { "Scan" };
    let header = row![
        column![
            text("PHP Versions").size(14).color(TEXT_SECONDARY),
            Space::with_height(3),
            text("Install / switch PHP via apt · enable Apache PHP module per version").size(11).color(TEXT_MUTED),
        ].spacing(0).width(Length::Fill),
        button(text(scan_lbl).size(12).color(TEAL))
            .on_press_maybe(if tab.scanning { None } else { Some(Message::Tools(ToolsMessage::ScanPhp)) })
            .padding(Padding::from([7, 14]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                    iced::widget::button::Style { background: Some(TEAL_HOVER.into()), text_color: TEAL,
                        border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
                _ => iced::widget::button::Style { background: Some(TEAL_BG.into()), text_color: TEAL,
                    border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
            }),
    ].align_y(Alignment::Center);

    let rows: Vec<Element<Message>> = tab.php_releases.iter().map(php_row).collect();

    container(column![
        header, Space::with_height(18), thin_line(), Space::with_height(14),
        container(row![
            Space::with_width(19 + 12),
            text("Version / apt status").size(10).color(TEXT_MUTED).width(Length::Fill),
            text("Apache mod").size(10).color(TEXT_MUTED).width(160),
            Space::with_width(12),
            text("apt action").size(10).color(TEXT_MUTED),
        ].align_y(Alignment::Center)).padding(Padding::from([0, 14])),
        Space::with_height(6),
        column(rows).spacing(8),
        Space::with_height(16),
        container(row![
            text("i").size(10).color(BLUE), Space::with_width(8),
            column![
                text("Requires ondrej/php PPA for multiple PHP versions.").size(11).color(TEXT_MUTED),
                Space::with_height(3),
                text("Apache mod: enables libapache2-mod-phpX.Y so Apache serves that PHP version directly.").size(11).color(TEXT_MUTED),
            ].spacing(0),
        ].align_y(Alignment::Start)).padding(Padding::from([10, 12])).width(Length::Fill)
        .style(|_: &iced::Theme| container::Style { background: Some(BLUE_BG.into()),
            border: Border { color: BLUE_BORDER, width: 1.0, radius: 8.0.into() }, ..Default::default() }),
    ].spacing(0).padding(Padding::from([22, 22]))).width(Length::Fill).style(card_style()).into()
}

fn php_row<'a>(r: &'a PhpRelease) -> Element<'a, Message> {
    let (status_color, status_label) = match r.status {
        PhpStatus::Installed => (GREEN, "Installed"),
        PhpStatus::Available => (TEXT_MUTED, "Available"),
        PhpStatus::Unknown => (TEXT_MUTED, "Unknown"),
    };
    let is_php56 = r.version == "5.6";

    let active_badge: Element<Message> = if r.is_active {
        container(text("Active").size(10).color(TEAL)).padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style { background: Some(TEAL_BG.into()),
                border: Border { radius: 20.0.into(), ..Default::default() }, ..Default::default() }).into()
    } else { Space::with_width(0).into() };

    let eol_badge: Element<Message> = if is_php56 {
        container(text("EOL").size(9).color(YELLOW)).padding(Padding::from([3, 7]))
            .style(|_: &iced::Theme| container::Style { background: Some(YELLOW_BG.into()),
                border: Border { color: YELLOW_BORDER, width: 1.0, radius: 20.0.into() }, ..Default::default() }).into()
    } else { Space::with_width(0).into() };

    let dot = status_dot(status_color);

    let apt_btn: Element<Message> = match r.status {
        PhpStatus::Installed => small_action_btn("Remove", RED, RED_BG, RED_HOVER,
            Message::Tools(ToolsMessage::RemovePhp(r.version.clone()))),
        _ => small_action_btn("Install", GREEN, GREEN_BG, GREEN_HOVER,
            Message::Tools(ToolsMessage::InstallPhp(r.version.clone()))),
    };

    let mod_name = format!("php{}", r.version);
    let (mod_dot_color, mod_status_lbl) = if r.apache_mod_available {
        if r.apache_mod_enabled { (GREEN, "enabled") } else { (YELLOW, "disabled") }
    } else { (BORDER_SUBTLE, "not available") };
    let mod_dot = status_dot(mod_dot_color);

    let apache_btn: Element<Message> = if r.apache_mod_available {
        if r.apache_mod_enabled {
            small_action_btn("Disable mod", RED, RED_BG, RED_HOVER,
                Message::Tools(ToolsMessage::DisableApacheMod(mod_name)))
        } else {
            small_action_btn("Enable mod", BLUE, BLUE_BG, BLUE_HOVER,
                Message::Tools(ToolsMessage::EnableApacheMod(mod_name)))
        }
    } else {
        container(text("no apache mod").size(10).color(TEXT_MUTED)).padding(Padding::from([6, 0])).into()
    };

    let card: Element<Message> = container(
        row![
            dot, Space::with_width(12),
            column![
                row![
                    text(format!("PHP {}", r.version)).size(14).color(TEXT_PRIMARY),
                    Space::with_width(8), active_badge, Space::with_width(4), eol_badge,
                ].align_y(Alignment::Center),
                Space::with_height(2),
                text(status_label).size(11).color(status_color),
            ].spacing(0).width(Length::Fill),
            container(Space::with_width(1)).width(1).height(34)
                .style(|_: &iced::Theme| container::Style { background: Some(BORDER_SUBTLE.into()), ..Default::default() }),
            Space::with_width(12),
            column![
                row![mod_dot, Space::with_width(6), text(mod_status_lbl).size(10).color(TEXT_MUTED)].align_y(Alignment::Center),
                Space::with_height(5), apache_btn,
            ].spacing(0).width(160),
            Space::with_width(12), apt_btn,
        ].align_y(Alignment::Center),
    ).padding(Padding::from([12, 14])).width(Length::Fill)
    .style(|_: &iced::Theme| container::Style { background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() }).into();

    let ppa_hint: Element<Message> = if is_php56 && r.status != PhpStatus::Installed {
        column![
            Space::with_height(4),
            container(row![
                text("!").size(9).color(YELLOW), Space::with_width(6),
                text("Requires ondrej/php PPA:  sudo add-apt-repository ppa:ondrej/php").size(10).color(TEXT_MUTED),
            ].align_y(Alignment::Center)).padding(Padding::from([6, 14])).width(Length::Fill)
            .style(|_: &iced::Theme| container::Style { background: Some(YELLOW_BG.into()),
                border: Border { color: YELLOW_BORDER, width: 1.0, radius: 6.0.into() }, ..Default::default() }),
        ].spacing(0).into()
    } else { Space::with_height(0).into() };

    column![card, ppa_hint].spacing(0).into()
}

fn apache_mods_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_lbl = if tab.mods_scanning { "Scanning…" } else { "Scan" };
    let header = row![
        column![
            text("Apache Modules").size(14).color(TEXT_SECONDARY),
            Space::with_height(3),
            text("All modules in /etc/apache2/mods-available/ — enable or disable via a2enmod / a2dismod").size(11).color(TEXT_MUTED),
        ].spacing(0).width(Length::Fill),
        button(text(scan_lbl).size(12).color(TEAL))
            .on_press_maybe(if tab.mods_scanning { None } else { Some(Message::Tools(ToolsMessage::ScanApacheMods)) })
            .padding(Padding::from([7, 14]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                    iced::widget::button::Style { background: Some(TEAL_HOVER.into()), text_color: TEAL,
                        border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
                _ => iced::widget::button::Style { background: Some(TEAL_BG.into()), text_color: TEAL,
                    border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
            }),
    ].align_y(Alignment::Center);

    let total = tab.apache_mods.len();
    let enabled = tab.apache_mods.iter().filter(|m| m.enabled).count();

    let filter_row = row![
        text_input("Filter modules…", &tab.mod_filter)
            .on_input(|v| Message::Tools(ToolsMessage::ModFilterChanged(v)))
            .padding(Padding::from([7, 12])).size(12)
            .style(|_, _| iced::widget::text_input::Style {
                background: BG_SURFACE.into(),
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
                icon: TEXT_MUTED, placeholder: TEXT_MUTED, value: TEXT_PRIMARY, selection: TEAL,
            }).width(Length::Fill),
    ];

    let q = tab.mod_filter.to_lowercase();
    let filtered: Vec<&ApacheModule> = tab.apache_mods.iter()
        .filter(|m| q.is_empty() || m.name.contains(&q)).collect();

    let body: Element<Message> = if total == 0 {
        container(column![
            text("No modules found.").size(13).color(TEXT_MUTED),
            Space::with_height(6),
            text("Click Scan to read /etc/apache2/mods-available/").size(11).color(TEXT_MUTED),
        ].spacing(0)).padding(Padding::from([20, 0])).into()
    } else {
        let rows: Vec<Element<Message>> = filtered.iter().map(|m| apache_mod_row(m)).collect();
        scrollable(column(rows).spacing(5)).height(420).into()
    };

    container(column![
        header, Space::with_height(14),
        if total > 0 {
            row![
                status_dot(GREEN), Space::with_width(6), text(format!("{} enabled", enabled)).size(11).color(TEXT_MUTED),
                Space::with_width(18),
                status_dot(BORDER_MED), Space::with_width(6), text(format!("{} disabled", total - enabled)).size(11).color(TEXT_MUTED),
                Space::with_width(18),
                text(format!("{} total", total)).size(11).color(TEXT_MUTED),
            ].align_y(Alignment::Center)
        } else { row![Space::with_width(0)] },
        Space::with_height(10), filter_row, Space::with_height(14), thin_line(), Space::with_height(10), body,
        Space::with_height(16),
        container(row![
            text("!").size(10).color(YELLOW), Space::with_width(8),
            text("Enabling/disabling modules requires sudo and reloads Apache automatically.").size(11).color(TEXT_MUTED),
        ].align_y(Alignment::Center)).padding(Padding::from([10, 12])).width(Length::Fill)
        .style(|_: &iced::Theme| container::Style { background: Some(YELLOW_BG.into()),
            border: Border { color: YELLOW_BORDER, width: 1.0, radius: 8.0.into() }, ..Default::default() }),
    ].spacing(0).padding(Padding::from([22, 22]))).width(Length::Fill).style(card_style()).into()
}

fn apache_mod_row<'a>(m: &'a ApacheModule) -> Element<'a, Message> {
    let (dot_color, status_text) = if m.enabled { (GREEN, "enabled") } else { (BORDER_MED, "disabled") };
    let action: Element<Message> = if m.enabled {
        small_action_btn("Disable", RED, RED_BG, RED_HOVER, Message::Tools(ToolsMessage::DisableApacheMod(m.name.clone())))
    } else {
        small_action_btn("Enable", GREEN, GREEN_BG, GREEN_HOVER, Message::Tools(ToolsMessage::EnableApacheMod(m.name.clone())))
    };
    container(row![
        status_dot(dot_color), Space::with_width(12),
        column![
            text(format!("mod_{}", m.name)).size(13).color(TEXT_PRIMARY),
            Space::with_height(2),
            text(status_text).size(11).color(dot_color),
        ].spacing(0).width(Length::Fill),
        action,
    ].align_y(Alignment::Center)).padding(Padding::from([10, 14])).width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(if m.enabled { BG_SURFACE } else { BG_BASE }.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }).into()
}

fn php_exts_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let active_ver: Option<String> = tab.php_releases.iter().find(|r| r.is_active).map(|r| r.version.clone());
    let ver_label = active_ver.as_deref().unwrap_or("active");

    let header = row![
        column![
            text("PHP Extensions").size(14).color(TEXT_SECONDARY),
            Space::with_height(3),
            text(format!("Install extensions for PHP {} via apt", ver_label)).size(11).color(TEXT_MUTED),
        ].spacing(0).width(Length::Fill),
        button(text("Scan").size(12).color(TEAL))
            .on_press(Message::Tools(ToolsMessage::ScanPhpExts))
            .padding(Padding::from([7, 14]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                    iced::widget::button::Style { background: Some(TEAL_HOVER.into()), text_color: TEAL,
                        border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
                _ => iced::widget::button::Style { background: Some(TEAL_BG.into()), text_color: TEAL,
                    border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
            }),
    ].align_y(Alignment::Center);

    let rows: Vec<Element<Message>> = tab.php_exts.iter().map(|e| php_ext_row(e, &active_ver)).collect();
    container(column![
        header, Space::with_height(18), thin_line(), Space::with_height(14),
        column(rows).spacing(8), Space::with_height(16),
        container(row![
            text("i").size(10).color(BLUE), Space::with_width(8),
            text("Extensions are installed for the currently active PHP version. Scan PHP Versions first.").size(11).color(TEXT_MUTED),
        ].align_y(Alignment::Center)).padding(Padding::from([10, 12])).width(Length::Fill)
        .style(|_: &iced::Theme| container::Style { background: Some(BLUE_BG.into()),
            border: Border { color: BLUE_BORDER, width: 1.0, radius: 8.0.into() }, ..Default::default() }),
    ].spacing(0).padding(Padding::from([22, 22]))).width(Length::Fill).style(card_style()).into()
}

fn php_ext_row<'a>(ext: &'a PhpExtension, active_ver: &Option<String>) -> Element<'a, Message> {
    let (dot_color, status_text) = if ext.installed { (GREEN, "Installed") } else { (TEXT_MUTED, "Not installed") };
    let pkg = match active_ver {
        Some(ver) => format!("php{}-{}", ver, ext.name),
        None => ext.pkg_suffix.clone(),
    };
    let action: Element<Message> = if ext.installed {
        small_action_btn("Remove", RED, RED_BG, RED_HOVER, Message::Tools(ToolsMessage::RemovePhpExt(pkg)))
    } else {
        small_action_btn("Install", GREEN, GREEN_BG, GREEN_HOVER, Message::Tools(ToolsMessage::InstallPhpExt(pkg)))
    };
    container(row![
        status_dot(dot_color), Space::with_width(12),
        column![
            row![
                text(ext.name.as_str()).size(13).color(TEXT_PRIMARY),
                Space::with_width(8),
                text(ext.pkg_suffix.as_str()).size(10).color(TEXT_MUTED),
            ].align_y(Alignment::Center),
            Space::with_height(2),
            text(status_text).size(11).color(dot_color),
        ].spacing(0).width(Length::Fill),
        action,
    ].align_y(Alignment::Center)).padding(Padding::from([12, 14])).width(Length::Fill)
    .style(|_: &iced::Theme| container::Style { background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() }).into()
}

fn db_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let note = container(row![
        text("").size(10).color(YELLOW), Space::with_width(8),
        text("Opens your system terminal as root.").size(11).color(TEXT_MUTED),
    ].align_y(Alignment::Center)).padding(Padding::from([10, 12])).width(Length::Fill)
    .style(|_: &iced::Theme| container::Style { background: Some(YELLOW_BG.into()),
        border: Border { color: YELLOW_BORDER, width: 1.0, radius: 8.0.into() }, ..Default::default() });

    let status_row: Element<Message> = if !tab.db_status.is_empty() {
        container(text(&tab.db_status).size(12).color(TEXT_SECONDARY))
            .padding(Padding::from([10, 12])).width(Length::Fill)
            .style(|_: &iced::Theme| container::Style { background: Some(BG_SURFACE.into()),
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() }).into()
    } else { Space::with_height(0).into() };

    container(column![
        text("Database CLI").size(14).color(TEXT_SECONDARY),
        Space::with_height(3),
        text("Launch a MySQL/MariaDB shell in your terminal").size(11).color(TEXT_MUTED),
        Space::with_height(18), thin_line(), Space::with_height(14),
        db_btn("MySQL / MariaDB", "Open root shell in terminal", BLUE, BLUE_BG, BLUE_HOVER, BLUE_BORDER,
            Message::Tools(ToolsMessage::OpenMysqlCli)),
        Space::with_height(8),
        db_btn("MariaDB (explicit)", "Forces mariadb binary if installed", PURPLE, PURPLE_BG, PURPLE_HOVER, PURPLE_BORDER,
            Message::Tools(ToolsMessage::OpenMariadbCli)),
        Space::with_height(8),
        db_btn("MySQL (socket auth)", "Connect via unix socket, no password", TEAL, TEAL_BG, TEAL_HOVER, TEAL_BORDER,
            Message::Tools(ToolsMessage::OpenMysqlSocket)),
        Space::with_height(16),
        status_row,
        if tab.db_status.is_empty() { Space::with_height(0) } else { Space::with_height(12) },
        note,
    ].spacing(0).padding(Padding::from([22, 22]))).width(Length::Fill).style(card_style()).into()
}

fn log_panel(tab: &ToolsTab) -> Element<'_, Message> {
    if tab.install_log.is_empty() { return Space::with_height(0).into(); }

    let rows: Vec<Element<Message>> = tab.install_log.iter().map(|(ok, msg)| {
        let (prefix, color) = if *ok { ("OK  ", GREEN) } else { ("ERR ", RED) };
        row![text(prefix).size(11).color(color), text(msg.as_str()).size(12).color(TEXT_SECONDARY)].into()
    }).collect();

    container(column![
        row![
            text("Activity Log").size(12).color(TEXT_MUTED).width(Length::Fill),
            button(text("Clear").size(11).color(TEXT_MUTED))
                .on_press(Message::Tools(ToolsMessage::ClearLog))
                .padding(Padding::from([4, 10]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(BG_HOVER.into()), text_color: TEXT_PRIMARY,
                        border: Border { radius: 6.0.into(), ..Default::default() }, ..Default::default() },
                    _ => iced::widget::button::Style { background: None, text_color: TEXT_MUTED, ..Default::default() },
                }),
        ].align_y(Alignment::Center),
        Space::with_height(10),
        scrollable(column(rows).spacing(5).padding(Padding::from([4, 0]))).height(150),
    ].spacing(0).padding(Padding::from([16, 18]))).width(Length::Fill)
    .style(|_: &iced::Theme| container::Style { background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() }).into()
}

fn error_suggestion_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let php_version = tab.install_log.iter().rev().find_map(|(ok, msg)| {
        if !*ok && msg.contains("PHP") {
            msg.split("PHP ").nth(1)
                .and_then(|s| s.split_whitespace().next())
                .map(|s| s.to_string())
        } else { None }
    }).unwrap_or_else(|| "8.2".to_string());

    let fix_commands = format!(
        "# Add the packages.sury.org/php repository.\n\
         sudo apt-get update\n\
         sudo apt-get install -y lsb-release ca-certificates apt-transport-https curl\n\
         sudo curl -sSLo /tmp/debsuryorg-archive-keyring.deb https://packages.sury.org/debsuryorg-archive-keyring.deb\n\
         sudo dpkg -i /tmp/debsuryorg-archive-keyring.deb\n\
         sudo sh -c 'echo \"deb [signed-by=/usr/share/keyrings/debsuryorg-archive-keyring.gpg] \
         https://packages.sury.org/php/ $(lsb_release -sc) main\" > /etc/apt/sources.list.d/php.list'\n\
         sudo apt-get update\n\n# Install PHP\nsudo apt-get install -y php{}",
        php_version
    );

    container(column![
        row![
            text("⚠ PHP Not Found").size(13).color(Color { r: 1.0, g: 0.650, b: 0.0, a: 1.0 }),
            Space::with_width(Length::Fill),
        ].align_y(Alignment::Center),
        Space::with_height(10),
        text("The ondrej/php PPA is not installed. Run these commands to add it:").size(12).color(TEXT_SECONDARY),
        Space::with_height(12),
        container(scrollable(text(fix_commands.clone()).size(10).color(BORDER_MED)).height(180))
            .padding(Padding::from([12, 14]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(Color { r: 0.08, g: 0.08, b: 0.08, a: 1.0 }.into()),
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }),
        Space::with_height(10),
        button(text("Get Text File").size(11).color(TEXT_PRIMARY))
            .on_press(Message::Tools(ToolsMessage::CopyFixCommands(fix_commands)))
            .padding(Padding::from([6, 12]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(BLUE_HOVER.into()), text_color: Color::WHITE,
                    border: Border { radius: 6.0.into(), ..Default::default() }, ..Default::default() },
                _ => iced::widget::button::Style { background: Some(BLUE_BG.into()), text_color: TEXT_PRIMARY,
                    border: Border { radius: 6.0.into(), ..Default::default() }, ..Default::default() },
            }),
    ].spacing(0)).width(Length::Fill).padding(Padding::from([16, 18]))
    .style(|_: &iced::Theme| container::Style {
        background: Some(Color { r: 0.200, g: 0.120, b: 0.080, a: 1.0 }.into()),
        border: Border { color: Color { r: 1.0, g: 0.650, b: 0.0, a: 1.0 }, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }).into()
}

fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(BG_CARD.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() },
        ..Default::default()
    }
}
fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1)).width(Length::Fill).height(1)
        .style(|_: &iced::Theme| container::Style { background: Some(BORDER_SUBTLE.into()), ..Default::default() })
}
fn status_dot(color: Color) -> iced::widget::Container<'static, Message> {
    container(Space::with_width(7)).width(7).height(7)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
}
fn small_action_btn<'a>(label: &'a str, color: Color, bg: Color, bg_hover: Color, msg: Message) -> Element<'a, Message> {
    button(text(label).size(12).color(color)).on_press(msg)
        .padding(Padding::from([6, 14]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style { background: Some(bg_hover.into()), text_color: color,
                    border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
            _ => iced::widget::button::Style { background: Some(bg.into()), text_color: color,
                border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default() },
        }).into()
}
fn db_btn<'a>(title: &'a str, subtitle: &'a str, accent: Color, bg: Color, bg_hover: Color, _border: Color, msg: Message) -> Element<'a, Message> {
    button(row![
        container(Space::with_width(4)).width(4).height(28)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(accent.into()),
                border: Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            }),
        Space::with_width(12),
        column![
            text(title).size(13).color(TEXT_PRIMARY),
            Space::with_height(2),
            text(subtitle).size(11).color(TEXT_MUTED),
        ].spacing(0).width(Length::Fill),
    ].align_y(Alignment::Center)).on_press(msg)
    .padding(Padding::from([12, 14])).width(Length::Fill)
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
            iced::widget::button::Style { background: Some(bg_hover.into()), text_color: TEXT_PRIMARY,
                border: Border { color: BORDER_MED, width: 1.0, radius: 8.0.into() }, ..Default::default() },
        _ => iced::widget::button::Style { background: Some(bg.into()), text_color: TEXT_PRIMARY,
            border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() },
    }).into()
}
