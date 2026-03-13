// src/tabs/vhosts/view.rs — all UI rendering for the VHosts tab

use super::{FormMode, VHostEntry, VHostView, VHostsTab};
use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

// ── Tinted solid backgrounds ──────────────────────────────────────────────

const GREEN_BG:   Color = Color { r: 0.050, g: 0.160, b: 0.090, a: 1.0 };
const GREEN_HOVER:Color = Color { r: 0.060, g: 0.185, b: 0.100, a: 1.0 };
const RED_BG:     Color = Color { r: 0.200, g: 0.060, b: 0.055, a: 1.0 };
const RED_HOVER:  Color = Color { r: 0.230, g: 0.070, b: 0.063, a: 1.0 };
const BLUE_BG:    Color = Color { r: 0.050, g: 0.090, b: 0.180, a: 1.0 };
const BLUE_HOVER: Color = Color { r: 0.070, g: 0.120, b: 0.230, a: 1.0 };
const BLUE_BORDER:Color = Color { r: 0.080, g: 0.140, b: 0.260, a: 1.0 };
const TEAL_BG:    Color = Color { r: 0.040, g: 0.160, b: 0.150, a: 1.0 };
const TEAL_HOVER: Color = Color { r: 0.050, g: 0.185, b: 0.175, a: 1.0 };
const TEAL_BORDER:Color = Color { r: 0.060, g: 0.210, b: 0.200, a: 1.0 };

// ── Entry point ───────────────────────────────────────────────────────────

pub fn render(tab: &VHostsTab) -> Element<'_, Message> {
    match tab.view_mode {
        VHostView::List         => list_view(tab),
        VHostView::ConfigEditor => config_editor_view(tab),
    }
}

// ── List view ─────────────────────────────────────────────────────────────

fn list_view(tab: &VHostsTab) -> Element<'_, Message> {
    let header = column![
        text("VirtualHosts").size(22).color(TEXT_PRIMARY),
        Space::with_height(4),
        text("All vhosts are stored in one file: devpanel.conf").size(13).color(TEXT_MUTED),
    ].spacing(0);

    let path_bar = container(row![
        column![
            text("Config file").size(10).color(TEXT_MUTED),
            Space::with_height(2),
            text(tab.devpanel_conf.as_str()).size(12).color(TEXT_SECONDARY),
        ].spacing(0).width(Length::Fill),
        icon_btn("Edit Config",  BLUE,  BLUE_BG,  BLUE_HOVER,  BLUE_BORDER, Some(Message::VH_OpenConfigEditor)),
        Space::with_width(8),
        icon_btn("Open File",    BLUE,  BLUE_BG,  BLUE_HOVER,  BLUE_BORDER, Some(Message::VH_OpenDevpanelConf)),
        Space::with_width(8),
        icon_btn(if tab.scanning { "Scanning…" } else { "Reload" }, TEAL, TEAL_BG, TEAL_HOVER, TEAL_BORDER,
            if tab.scanning { None } else { Some(Message::VH_Scan) }),
        Space::with_width(8),
        icon_btn("+ Add VHost",  GREEN, GREEN_BG, GREEN_HOVER, GREEN_BG,    Some(Message::VH_ShowAddForm)),
    ].align_y(Alignment::Center))
    .padding(Padding::from([12, 16])).width(Length::Fill).style(surface_style());

    let form_el: Element<Message> = if tab.form.mode == FormMode::Hidden {
        Space::with_height(0).into()
    } else {
        add_form_widget(tab)
    };

    let status: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => {
            let (color, bg) = if *ok { (GREEN, GREEN_BG) } else { (RED, RED_BG) };
            container(row![
                container(Space::with_width(6)).width(6).height(6)
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(color.into()),
                        border: Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                Space::with_width(8),
                text(msg.as_str()).size(12).color(TEXT_SECONDARY),
            ].align_y(Alignment::Center))
            .padding(Padding::from([10, 14])).width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()),
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            }).into()
        }
        None => Space::with_height(0).into(),
    };

    let body: Element<Message> = if tab.vhosts.is_empty() && !tab.scanning {
        container(column![
            text("No virtual hosts found in devpanel.conf").size(15).color(TEXT_SECONDARY),
            Space::with_height(8),
            text("Click \"+ Add VHost\" to create your first one").size(13).color(TEXT_MUTED),
        ].align_x(Alignment::Center))
        .width(Length::Fill).padding(Padding::from([40, 0])).center_x(Length::Fill).into()
    } else if tab.scanning {
        container(text("Scanning…").size(14).color(TEXT_MUTED))
            .width(Length::Fill).padding(Padding::from([40, 0])).center_x(Length::Fill).into()
    } else {
        column(tab.vhosts.iter().map(|v| vhost_row(tab, v)).collect::<Vec<_>>())
            .spacing(8).into()
    };

    scrollable(column![
        header,
        Space::with_height(18),
        path_bar,
        Space::with_height(10),
        form_el,
        if tab.form.mode != FormMode::Hidden { Space::with_height(10) } else { Space::with_height(0) },
        status,
        if tab.status_msg.is_some() { Space::with_height(12) } else { Space::with_height(0) },
        body,
        Space::with_height(24),
    ].spacing(0).padding(Padding::from([22, 24]))).into()
}

// ── Config editor view ────────────────────────────────────────────────────

fn config_editor_view(tab: &VHostsTab) -> Element<'_, Message> {
    let header = row![
        column![
            text("Config Editor").size(22).color(TEXT_PRIMARY),
            Space::with_height(4),
            text(tab.devpanel_conf.as_str()).size(12).color(TEXT_MUTED),
        ].spacing(0).width(Length::Fill),
        icon_btn("← Back", TEAL, TEAL_BG, TEAL_HOVER, TEAL_BORDER, Some(Message::VH_CloseConfigEditor)),
        Space::with_width(8),
        icon_btn(
            if tab.config_loading { "Saving…" } else { "Save" },
            GREEN, GREEN_BG, GREEN_HOVER, GREEN_BG,
            if tab.config_loading { None } else { Some(Message::VH_SaveConfigFile) },
        ),
    ].align_y(Alignment::Center);

    let dirty_badge: Element<Message> = if tab.config_dirty {
        container(text("unsaved changes").size(10).color(YELLOW))
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(Color { r: 0.19, g: 0.16, b: 0.04, a: 1.0 }.into()),
                border: Border { radius: 20.0.into(), ..Default::default() },
                ..Default::default()
            }).into()
    } else {
        Space::with_height(0).into()
    };

    let editor = text_editor(&tab.config_content)
        .on_action(Message::VH_ConfigEditorAction)
        .height(Length::Fill)
        .padding(Padding::from([12, 14]));

    let editor_container = container(editor)
        .width(Length::Fill).height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_CARD.into()),
            border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
            ..Default::default()
        });

    column![
        Space::with_height(22),
        container(column![
            header, Space::with_height(8), dirty_badge,
        ].spacing(0).padding(Padding::from([0, 24]))).width(Length::Fill),
        Space::with_height(12),
        container(editor_container).width(Length::Fill).height(Length::Fill).padding(Padding::from([0, 24])),
        Space::with_height(16),
    ].height(Length::Fill).into()
}

// ── VHost row (with inline edit) ──────────────────────────────────────────

fn vhost_row<'a>(tab: &'a VHostsTab, vh: &'a VHostEntry) -> Element<'a, Message> {
    let idx = vh.index;

    // Inline edit: replace the card with the edit form in-place
    if matches!(tab.form.mode, FormMode::Edit(i) if i == idx) {
        return inline_edit_widget(tab, idx);
    }

    let sn = vh.server_name.clone();

    let name_row = row![
        text(vh.server_name.as_str()).size(14).color(TEXT_PRIMARY),
        Space::with_width(Length::Fill),
        container(text("active").size(10).color(GREEN))
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(GREEN_BG.into()),
                border: Border { radius: 20.0.into(), ..Default::default() },
                ..Default::default()
            }),
    ].align_y(Alignment::Center);

    let info_row = row![
        text("DocumentRoot").size(10).color(TEXT_MUTED),
        Space::with_width(8),
        text(if vh.document_root.is_empty() { "—" } else { vh.document_root.as_str() })
            .size(12).color(TEXT_SECONDARY),
    ].align_y(Alignment::Center);

    let is_confirming = tab.confirm_delete == Some(idx);
    let del_btn: Element<Message> = if is_confirming {
        row![
            small_btn("Confirm Delete", RED, RED_BG, RED_HOVER, RED_BG, Some(Message::VH_DeleteConfirm(idx))),
            Space::with_width(6),
            small_btn("Cancel", TEXT_MUTED, BG_SURFACE, BG_HOVER, BORDER_SUBTLE, Some(Message::VH_DeleteCancel)),
        ].align_y(Alignment::Center).into()
    } else {
        small_btn("Delete", RED, RED_BG, RED_HOVER, RED_BG, Some(Message::VH_DeleteRequest(idx)))
    };

    container(column![
        name_row, Space::with_height(6), info_row, Space::with_height(14), thin_line(), Space::with_height(12),
        row![
            small_btn("Edit",    BLUE, BLUE_BG, BLUE_HOVER, BLUE_BORDER, Some(Message::VH_EditRequest(idx))),
            Space::with_width(6),
            small_btn("Browser", TEAL, TEAL_BG, TEAL_HOVER, TEAL_BORDER, Some(Message::VH_OpenBrowser(sn))),
            Space::with_width(Length::Fill),
            del_btn,
        ].align_y(Alignment::Center),
    ].spacing(0))
    .padding(Padding::from([16, 18])).width(Length::Fill).style(card_style()).into()
}

fn inline_edit_widget<'a>(tab: &'a VHostsTab, _idx: usize) -> Element<'a, Message> {
    let can_save = !tab.form.server_name.trim().is_empty() && !tab.form.document_root.trim().is_empty();

    let submit_btn = button(text("Save Changes").size(13).color(if can_save { GREEN } else { TEXT_MUTED }))
        .padding(Padding::from([9, 18]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered if can_save => iced::widget::button::Style {
                background: Some(GREEN_HOVER.into()), text_color: GREEN,
                border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default()
            },
            _ => iced::widget::button::Style {
                background: Some(if can_save { GREEN_BG } else { BG_SURFACE }.into()),
                text_color: if can_save { GREEN } else { TEXT_MUTED },
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default()
            },
        });
    let submit_el: Element<Message> = if can_save { submit_btn.on_press(Message::VH_SaveEdit).into() } else { submit_btn.into() };

    container(column![
        row![
            text("Editing VirtualHost").size(13).color(BLUE),
            Space::with_width(Length::Fill),
            small_btn("Cancel", TEXT_MUTED, BG_SURFACE, BG_HOVER, BORDER_SUBTLE, Some(Message::VH_HideForm)),
        ].align_y(Alignment::Center),
        Space::with_height(14), thin_line(), Space::with_height(14),
        row![
            column![
                text("ServerName").size(11).color(TEXT_MUTED),
                Space::with_height(5),
                text_input("myproject.local", &tab.form.server_name)
                    .on_input(Message::VH_FormServerNameChanged).size(13).padding(Padding::from([8, 10])).width(Length::Fill),
            ].spacing(0).width(Length::FillPortion(1)),
            Space::with_width(14),
            column![
                text("DocumentRoot").size(11).color(TEXT_MUTED),
                Space::with_height(5),
                text_input("/home/user/projects/app/public", &tab.form.document_root)
                    .on_input(Message::VH_FormDocRootChanged).size(13).padding(Padding::from([8, 10])).width(Length::Fill),
            ].spacing(0).width(Length::FillPortion(2)),
        ].align_y(Alignment::Start),
        Space::with_height(14),
        submit_el,
    ].spacing(0).padding(Padding::from([16, 18]))).width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border { color: BLUE_BORDER, width: 1.5, radius: 10.0.into() },
        ..Default::default()
    }).into()
}

fn add_form_widget(tab: &VHostsTab) -> Element<'_, Message> {
    let is_edit   = matches!(tab.form.mode, FormMode::Edit(_));
    let can_save  = !tab.form.server_name.trim().is_empty() && !tab.form.document_root.trim().is_empty();
    let save_msg  = if is_edit { Message::VH_SaveEdit } else { Message::VH_Create };
    let save_lbl  = if is_edit { "Save Changes" } else { "Create VirtualHost" };

    let submit_btn = button(text(save_lbl).size(13).color(if can_save { GREEN } else { TEXT_MUTED }))
        .padding(Padding::from([9, 18]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered if can_save => iced::widget::button::Style {
                background: Some(GREEN_HOVER.into()), text_color: GREEN,
                border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default()
            },
            _ => iced::widget::button::Style {
                background: Some(if can_save { GREEN_BG } else { BG_SURFACE }.into()),
                text_color: if can_save { GREEN } else { TEXT_MUTED },
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default()
            },
        });
    let submit_el: Element<Message> = if can_save { submit_btn.on_press(save_msg).into() } else { submit_btn.into() };

    container(column![
        row![
            text(if is_edit { "Edit VirtualHost" } else { "Add VirtualHost" }).size(14).color(TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            small_btn("Cancel", TEXT_MUTED, BG_SURFACE, BG_HOVER, BORDER_SUBTLE, Some(Message::VH_HideForm)),
        ].align_y(Alignment::Center),
        Space::with_height(16), thin_line(), Space::with_height(16),
        column![
            text("ServerName  (e.g. myproject.local)").size(11).color(TEXT_MUTED),
            Space::with_height(5),
            text_input("myproject.local", &tab.form.server_name)
                .on_input(Message::VH_FormServerNameChanged).size(13).padding(Padding::from([8, 10])).width(Length::Fill),
        ].spacing(0),
        Space::with_height(12),
        column![
            text("DocumentRoot  (full path, e.g. /home/user/projects/myapp/public)").size(11).color(TEXT_MUTED),
            Space::with_height(5),
            text_input("/home/user/projects/myapp/public", &tab.form.document_root)
                .on_input(Message::VH_FormDocRootChanged).size(13).padding(Padding::from([8, 10])).width(Length::Fill),
        ].spacing(0),
        Space::with_height(18),
        submit_el,
    ].spacing(0).padding(Padding::from([20, 22]))).width(Length::Fill).style(card_style()).into()
}

// ── Style helpers ─────────────────────────────────────────────────────────

fn card_style()    -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style { background: Some(BG_CARD.into()),    border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() }, ..Default::default() }
}
fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style { background: Some(BG_SURFACE.into()), border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into()  }, ..Default::default() }
}
fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1)).width(Length::Fill).height(1)
        .style(|_: &iced::Theme| container::Style { background: Some(BORDER_SUBTLE.into()), ..Default::default() })
}
fn small_btn<'a>(label: &'a str, color: Color, bg: Color, bg_hover: Color, border: Color, on_press: Option<Message>) -> Element<'a, Message> {
    let b = button(text(label).size(12).color(color))
        .padding(Padding::from([6, 12]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style { background: Some(bg_hover.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 7.0.into() }, ..Default::default() },
            _ =>
                iced::widget::button::Style { background: Some(bg.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 7.0.into() }, ..Default::default() },
        });
    if let Some(msg) = on_press { b.on_press(msg).into() } else { b.into() }
}
fn icon_btn<'a>(label: &'a str, color: Color, bg: Color, bg_hover: Color, border: Color, on_press: Option<Message>) -> Element<'a, Message> {
    let b = button(text(label).size(12).color(color))
        .padding(Padding::from([7, 14]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style { background: Some(bg_hover.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 8.0.into() }, ..Default::default() },
            _ =>
                iced::widget::button::Style { background: Some(bg.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 8.0.into() }, ..Default::default() },
        });
    if let Some(msg) = on_press { b.on_press(msg).into() } else { b.into() }
}
