use crate::ui::utils::styles;
use iced::widget::pick_list;
use iced::{Element, Length, Padding};
use std::borrow::Borrow;

pub fn dropdown<'a, T, L, V, Message>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
{
    dropdown_width(options, selected, on_selected, Length::Fill)
}

pub fn dropdown_width<'a, T, L, V, Message>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
    width: Length,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
{
    pick_list(options, selected, on_selected)
        .padding(Padding::from([10, 14]))
        .width(width)
        .style(styles::pick_list_style)
        .menu_style(styles::pick_list_menu_style)
        .text_size(13)
        .into()
}
