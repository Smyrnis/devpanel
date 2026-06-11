mod controls;
mod modal;
mod panels;
mod rows;

use crate::domain::dashboard::DashboardService;
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::Message;
use crate::ui::tabs::dashboard::DashboardTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, scrollable};
use iced::{Element, Padding};

pub fn render(tab: &DashboardTab, compact: bool) -> Element<'_, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let header = header_fn(tr(keys::TITLE), tr(keys::SUBTITLE), vec![]);

    let service_rows = ui::row_group(vec![
        rows::service_block(tab, DashboardService::Apache),
        rows::service_block(tab, DashboardService::MySql),
        rows::service_block(tab, DashboardService::Php),
    ]);
    let runtime_rows = ui::row_group(vec![
        rows::service_block(tab, DashboardService::Composer),
        rows::service_block(tab, DashboardService::Node),
    ]);

    let content = scrollable(
        column![
            header,
            Space::with_height(18),
            service_rows,
            Space::with_height(10),
            runtime_rows,
            Space::with_height(24)
        ]
        .spacing(0)
        .padding(Padding::from([20, 22])),
    );

    if tab.php_info_loading || tab.php_info.is_some() {
        column![content, modal::php_info_modal(tab)]
            .spacing(0)
            .into()
    } else {
        content.into()
    }
}
