// src/tabs/vhosts.rs — all vhosts live in /etc/apache2/sites-available/devpanel.conf

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

const GREEN_BG:      Color = Color { r: 0.050, g: 0.160, b: 0.090, a: 1.0 };
const GREEN_HOVER:   Color = Color { r: 0.060, g: 0.185, b: 0.100, a: 1.0 };
const RED_BG:        Color = Color { r: 0.200, g: 0.060, b: 0.055, a: 1.0 };
const RED_HOVER:     Color = Color { r: 0.230, g: 0.070, b: 0.063, a: 1.0 };
const BLUE_BG:       Color = Color { r: 0.050, g: 0.090, b: 0.180, a: 1.0 };
const BLUE_HOVER:    Color = Color { r: 0.070, g: 0.120, b: 0.230, a: 1.0 };
const BLUE_BORDER:   Color = Color { r: 0.080, g: 0.140, b: 0.260, a: 1.0 };
const TEAL_BG:       Color = Color { r: 0.040, g: 0.160, b: 0.150, a: 1.0 };
const TEAL_HOVER:    Color = Color { r: 0.050, g: 0.185, b: 0.175, a: 1.0 };
const TEAL_BORDER:   Color = Color { r: 0.060, g: 0.210, b: 0.200, a: 1.0 };

// ── A single parsed vhost from devpanel.conf ──────────────────────────────

#[derive(Debug, Clone)]
pub struct VHostEntry {
    pub server_name:   String,
    pub document_root: String,
    pub index:         usize,
}

// ── Form state ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FormMode { Hidden, Add, Edit(usize) }

#[derive(Debug, Clone)]
pub struct VHostForm {
    pub mode:          FormMode,
    pub server_name:   String,
    pub document_root: String,
}
impl VHostForm {
    pub fn new() -> Self { Self { mode: FormMode::Hidden, server_name: String::new(), document_root: String::new() } }
    pub fn open_add(&mut self)                { self.mode = FormMode::Add; self.server_name.clear(); self.document_root.clear(); }
    pub fn open_edit(&mut self, e: &VHostEntry) { self.mode = FormMode::Edit(e.index); self.server_name = e.server_name.clone(); self.document_root = e.document_root.clone(); }
    pub fn hide(&mut self)                    { self.mode = FormMode::Hidden; }
}

// ── Tab state ─────────────────────────────────────────────────────────────

pub struct VHostsTab {
    pub devpanel_conf:  String,
    pub vhosts:         Vec<VHostEntry>,
    pub scanning:       bool,
    pub form:           VHostForm,
    pub status_msg:     Option<(bool, String)>,
    pub confirm_delete: Option<usize>,
}

impl VHostsTab {
    pub fn new(devpanel_conf: String) -> Self {
        Self { devpanel_conf, vhosts: Vec::new(), scanning: false, form: VHostForm::new(), status_msg: None, confirm_delete: None }
    }
    pub fn set_vhosts(&mut self, v: Vec<VHostEntry>) { self.scanning = false; self.vhosts = v; }

    pub fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("VirtualHosts").size(22).color(TEXT_PRIMARY),
            Space::with_height(4),
            text("All vhosts are stored in one file: devpanel.conf").size(13).color(TEXT_MUTED),
        ].spacing(0);

        let path_bar = container(row![
            column![
                text("Config file").size(10).color(TEXT_MUTED),
                Space::with_height(2),
                text(self.devpanel_conf.as_str()).size(12).color(TEXT_SECONDARY),
            ].spacing(0).width(Length::Fill),
            icon_btn("Open File", BLUE, BLUE_BG, BLUE_HOVER, BLUE_BORDER, Some(Message::VH_OpenDevpanelConf)),
            Space::with_width(8),
            icon_btn(if self.scanning { "Scanning…" } else { "Reload" }, TEAL, TEAL_BG, TEAL_HOVER, TEAL_BORDER,
                if self.scanning { None } else { Some(Message::VH_Scan) }),
            Space::with_width(8),
            icon_btn("+ Add VHost", GREEN, GREEN_BG, GREEN_HOVER, GREEN_BG, Some(Message::VH_ShowAddForm)),
        ].align_y(Alignment::Center))
        .padding(Padding::from([12, 16])).width(Length::Fill).style(surface_style());

        let form_el: Element<Message> = if self.form.mode == FormMode::Hidden { Space::with_height(0).into() } else { self.form_widget() };

        let status: Element<Message> = if let Some((ok, msg)) = &self.status_msg {
            let (color, bg) = if *ok { (GREEN, GREEN_BG) } else { (RED, RED_BG) };
            container(row![
                container(Space::with_width(6)).width(6).height(6).style(move |_: &iced::Theme| container::Style {
                    background: Some(color.into()), border: Border { radius: 3.0.into(), ..Default::default() }, ..Default::default()
                }),
                Space::with_width(8), text(msg.as_str()).size(12).color(TEXT_SECONDARY),
            ].align_y(Alignment::Center))
            .padding(Padding::from([10, 14])).width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()), border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default()
            }).into()
        } else { Space::with_height(0).into() };

        let body: Element<Message> = if self.vhosts.is_empty() && !self.scanning {
            container(column![
                text("No virtual hosts found in devpanel.conf").size(15).color(TEXT_SECONDARY),
                Space::with_height(8),
                text("Click \"+ Add VHost\" to create your first one").size(13).color(TEXT_MUTED),
            ].align_x(Alignment::Center)).width(Length::Fill).padding(Padding::from([40,0])).center_x(Length::Fill).into()
        } else if self.scanning {
            container(text("Scanning…").size(14).color(TEXT_MUTED)).width(Length::Fill).padding(Padding::from([40,0])).center_x(Length::Fill).into()
        } else {
            column(self.vhosts.iter().map(|v| self.vhost_row(v)).collect::<Vec<_>>()).spacing(8).into()
        };

        scrollable(column![
            header, Space::with_height(18),
            path_bar, Space::with_height(10),
            form_el,
            if self.form.mode != FormMode::Hidden { Space::with_height(10) } else { Space::with_height(0) },
            status,
            if self.status_msg.is_some() { Space::with_height(12) } else { Space::with_height(0) },
            body, Space::with_height(24),
        ].spacing(0).padding(Padding::from([22, 24]))).into()
    }

    fn vhost_row<'a>(&self, vh: &'a VHostEntry) -> Element<'a, Message> {
        let idx = vh.index;
        let sn  = vh.server_name.clone();

        let name_row = row![
            text(vh.server_name.as_str()).size(14).color(TEXT_PRIMARY),
            Space::with_width(Length::Fill),
            container(text("active").size(10).color(GREEN))
                .padding(Padding::from([3,8]))
                .style(|_: &iced::Theme| container::Style { background: Some(GREEN_BG.into()), border: Border { radius: 20.0.into(), ..Default::default() }, ..Default::default() }),
        ].align_y(Alignment::Center);

        let info_row = row![
            text("DocumentRoot").size(10).color(TEXT_MUTED), Space::with_width(8),
            text(if vh.document_root.is_empty() { "—" } else { vh.document_root.as_str() }).size(12).color(TEXT_SECONDARY),
        ].align_y(Alignment::Center);

        let is_confirming = self.confirm_delete == Some(idx);
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
                small_btn("Edit", BLUE, BLUE_BG, BLUE_HOVER, BLUE_BORDER, Some(Message::VH_EditRequest(idx))),
                Space::with_width(6),
                small_btn("Browser", TEAL, TEAL_BG, TEAL_HOVER, TEAL_BORDER, Some(Message::VH_OpenBrowser(sn))),
                Space::with_width(Length::Fill),
                del_btn,
            ].align_y(Alignment::Center),
        ].spacing(0))
        .padding(Padding::from([16,18])).width(Length::Fill).style(card_style()).into()
    }

    fn form_widget(&self) -> Element<'_, Message> {
        let is_edit   = matches!(self.form.mode, FormMode::Edit(_));
        let can_save  = !self.form.server_name.trim().is_empty() && !self.form.document_root.trim().is_empty();
        let save_msg  = if is_edit { Message::VH_SaveEdit } else { Message::VH_Create };
        let save_lbl  = if is_edit { "Save Changes" } else { "Create VirtualHost" };

        let submit_btn = button(text(save_lbl).size(13).color(if can_save { GREEN } else { TEXT_MUTED }))
            .padding(Padding::from([9,18]))
            .style(move |_, status| match status {
                iced::widget::button::Status::Hovered if can_save => iced::widget::button::Style {
                    background: Some(GREEN_HOVER.into()), text_color: GREEN, border: Border { radius: 8.0.into(), ..Default::default() }, ..Default::default()
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
                text_input("myproject.local", &self.form.server_name)
                    .on_input(Message::VH_FormServerNameChanged).size(13).padding(Padding::from([8,10])).width(Length::Fill),
            ].spacing(0),
            Space::with_height(12),
            column![
                text("DocumentRoot  (full path, e.g. /home/user/projects/myapp/public)").size(11).color(TEXT_MUTED),
                Space::with_height(5),
                text_input("/home/user/projects/myapp/public", &self.form.document_root)
                    .on_input(Message::VH_FormDocRootChanged).size(13).padding(Padding::from([8,10])).width(Length::Fill),
            ].spacing(0),
            Space::with_height(18), submit_el,
        ].spacing(0).padding(Padding::from([20,22]))).width(Length::Fill).style(card_style()).into()
    }
}

// ── Styles ────────────────────────────────────────────────────────────────

fn card_style()    -> impl Fn(&iced::Theme) -> container::Style { |_| container::Style { background: Some(BG_CARD.into()),    border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() }, ..Default::default() } }
fn surface_style() -> impl Fn(&iced::Theme) -> container::Style { |_| container::Style { background: Some(BG_SURFACE.into()), border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into()  }, ..Default::default() } }
fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1)).width(Length::Fill).height(1).style(|_: &iced::Theme| container::Style { background: Some(BORDER_SUBTLE.into()), ..Default::default() })
}
fn small_btn<'a>(label: &'a str, color: Color, bg: Color, bg_hover: Color, border: Color, on_press: Option<Message>) -> Element<'a, Message> {
    let b = button(text(label).size(12).color(color)).padding(Padding::from([6,12])).style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => iced::widget::button::Style { background: Some(bg_hover.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 7.0.into() }, ..Default::default() },
        _ => iced::widget::button::Style { background: Some(bg.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 7.0.into() }, ..Default::default() },
    });
    if let Some(msg) = on_press { b.on_press(msg).into() } else { b.into() }
}
fn icon_btn<'a>(label: &'a str, color: Color, bg: Color, bg_hover: Color, border: Color, on_press: Option<Message>) -> Element<'a, Message> {
    let b = button(text(label).size(12).color(color)).padding(Padding::from([7,14])).style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => iced::widget::button::Style { background: Some(bg_hover.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 8.0.into() }, ..Default::default() },
        _ => iced::widget::button::Style { background: Some(bg.into()), text_color: color, border: Border { color: border, width: 1.0, radius: 8.0.into() }, ..Default::default() },
    });
    if let Some(msg) = on_press { b.on_press(msg).into() } else { b.into() }
}

// ── Parse / Build helpers ─────────────────────────────────────────────────

fn parse_directive(content: &str, directive: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if t.to_lowercase().starts_with(&directive.to_lowercase()) {
            let rest = &t[directive.len()..];
            return rest.trim().split_whitespace().next().unwrap_or("").to_string();
        }
    }
    String::new()
}

pub fn parse_vhosts_from_content(content: &str) -> Vec<VHostEntry> {
    let mut entries = Vec::new();
    let mut idx = 0usize;
    let mut in_block = false;
    let mut sn = String::new();
    let mut dr = String::new();
    for line in content.lines() {
        let t = line.trim().to_lowercase();
        if t.starts_with("<virtualhost") {
            in_block = true; sn.clear(); dr.clear();
        } else if t.starts_with("</virtualhost>") && in_block {
            if !sn.is_empty() { entries.push(VHostEntry { server_name: sn.clone(), document_root: dr.clone(), index: idx }); idx += 1; }
            in_block = false;
        } else if in_block {
            let orig = line.trim();
            if orig.to_lowercase().starts_with("servername")  { sn = parse_directive(orig, "ServerName"); }
            if orig.to_lowercase().starts_with("documentroot") { dr = parse_directive(orig, "DocumentRoot"); }
        }
    }
    entries
}

pub fn build_conf_content(entries: &[VHostEntry]) -> String {
    let mut out = String::from("# DevPanel managed VirtualHosts\n# Managed by DevPanel — use the UI to add/edit/remove entries.\n\n");
    for e in entries {
        let sn   = e.server_name.trim_end_matches('/');
        let slug = sn.replace('.', "_");
        out.push_str(&format!(
            "<VirtualHost *:80>\n    ServerName {sn}\n    ServerAlias www.{sn}\n    DocumentRoot {dr}\n\n    <Directory {dr}>\n        Options Indexes FollowSymLinks\n        AllowOverride All\n        Require all granted\n    </Directory>\n\n    ErrorLog ${{APACHE_LOG_DIR}}/{slug}_error.log\n    CustomLog ${{APACHE_LOG_DIR}}/{slug}_access.log combined\n</VirtualHost>\n\n",
            sn=sn, dr=e.document_root, slug=slug,
        ));
    }
    out
}

// ── Async tasks ───────────────────────────────────────────────────────────

pub async fn scan_vhosts(devpanel_conf: String) -> Vec<VHostEntry> {
    let content = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    parse_vhosts_from_content(&content)
}

async fn write_conf(path: &str, content: &str, password: &str) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let mut child = tokio::process::Command::new("sudo")
        .args(["-S", "tee", path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn().map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(format!("{}\n", password).as_bytes()).await;
        let _ = stdin.write_all(content.as_bytes()).await;
    }
    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).to_string()) }
}

pub async fn add_vhost(devpanel_conf: String, server_name: String, document_root: String, password: String) -> (bool, String) {
    let existing = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    let sn = server_name.trim().to_string();
    let dr = document_root.trim().to_string();
    if entries.iter().any(|e| e.server_name == sn) {
        return (false, format!("VirtualHost '{}' already exists", sn));
    }
    entries.push(VHostEntry { server_name: sn.clone(), document_root: dr, index: entries.len() });
    if let Err(e) = write_conf(&devpanel_conf, &build_conf_content(&entries), &password).await { return (false, format!("Write failed: {}", e)); }
    let hosts = tokio::fs::read_to_string("/etc/hosts").await.unwrap_or_default();
    if !hosts.contains(&sn) {
        let _ = crate::sudo_prompt::sudo_tee_append_with_password(&password, "/etc/hosts", &format!("127.0.0.1    {}\n", sn)).await;
    }
    let _ = crate::sudo_prompt::sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await;
    (true, format!("VirtualHost '{}' created and Apache reloaded", sn))
}

pub async fn edit_vhost(devpanel_conf: String, index: usize, server_name: String, document_root: String, password: String) -> (bool, String) {
    let existing = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() { return (false, "Index out of range".into()); }
    let old_sn = entries[index].server_name.clone();
    let new_sn = server_name.trim().to_string();
    entries[index].server_name   = new_sn.clone();
    entries[index].document_root = document_root.trim().to_string();
    if let Err(e) = write_conf(&devpanel_conf, &build_conf_content(&entries), &password).await { return (false, format!("Write failed: {}", e)); }
    if old_sn != new_sn {
        let hosts = tokio::fs::read_to_string("/etc/hosts").await.unwrap_or_default();
        if !hosts.contains(&new_sn) {
            let _ = crate::sudo_prompt::sudo_tee_append_with_password(&password, "/etc/hosts", &format!("127.0.0.1    {}\n", new_sn)).await;
        }
    }
    let _ = crate::sudo_prompt::sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await;
    (true, format!("VirtualHost '{}' updated", new_sn))
}

pub async fn delete_vhost(devpanel_conf: String, index: usize, password: String) -> (bool, String) {
    let existing = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() { return (false, "Index out of range".into()); }
    let removed = entries[index].server_name.clone();
    entries.remove(index);
    for (i, e) in entries.iter_mut().enumerate() { e.index = i; }
    if let Err(e) = write_conf(&devpanel_conf, &build_conf_content(&entries), &password).await { return (false, format!("Write failed: {}", e)); }
    let _ = crate::sudo_prompt::sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await;
    (true, format!("VirtualHost '{}' removed", removed))
}
