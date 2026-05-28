use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::{FirstRunMessage, Message};
use crate::ui::templates::prelude as ui;
use iced::Element;

pub(super) fn continue_button<'a>(installing: bool) -> Element<'a, Message> {
    ui::primary_text_button_maybe(
        if installing {
            tr(keys::INSTALLING)
        } else {
            tr(keys::CONTINUE_INSTALL)
        },
        if installing {
            None
        } else {
            Some(Message::FirstRun(FirstRunMessage::Continue))
        },
    )
}

pub(super) fn exit_button<'a>(installing: bool) -> Element<'a, Message> {
    ui::ghost_text_button_maybe(
        tr(keys::EXIT),
        if installing {
            None
        } else {
            Some(Message::FirstRun(FirstRunMessage::Exit))
        },
    )
}
