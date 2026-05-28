mod context;
mod sidebar;

use crate::app::App;
use crate::core::{
    first_run::FirstRunState,
    theme::{self, theme_map as theme_keys},
};
use crate::messages::{Message, Tab};
use iced::widget::{column, container, row, stack};
use iced::{Element, Length};

pub(super) const SIDEBAR_COLLAPSE_WIDTH: f32 = 786.0;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        if self.first_run_state == FirstRunState::Visible {
            return crate::installer::window::view(
                self.first_run_options,
                self.first_run_status,
                self.first_run_expanded,
                self.first_run_installing,
                &self.first_run_log_lines,
            );
        }

        let compact = self.is_compact();
        let tab_content: Element<Message> = match &self.active_tab {
            Tab::Dashboard => self.dashboard.view(compact),
            Tab::Tools => self.tools.view(compact),
            Tab::VHosts => self.vhosts.view(compact),
            Tab::Config => self.config_tab.view(&self.ssh_keys, compact),
        };

        let content_area = column![
            self.context_bar(),
            container(tab_content).width(Length::Fill)
        ]
        .height(Length::Fill)
        .spacing(0);

        let app_area = row![
            self.sidebar(),
            container(content_area)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::BG_BASE).into()),
                    ..Default::default()
                }),
        ];

        let base = stack![
            container(app_area).width(Length::Fill).height(Length::Fill),
            self.notification_overlay(),
        ];

        if self.sudo.is_visible() {
            stack![base, self.sudo.view()].into()
        } else {
            base.into()
        }
    }

    pub(crate) fn is_compact(&self) -> bool {
        self.window_size.width < SIDEBAR_COLLAPSE_WIDTH
    }
}
