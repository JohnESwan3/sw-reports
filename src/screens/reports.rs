use iced::widget::{column, container, text};
use iced::Element;

use crate::message::Message;

pub fn view<'a>(_collapsed: bool) -> Element<'a, Message> {
    container(
        column![
            text("Reports").size(28),
            text("Coming Soon")
        ]
        .spacing(12),
    )
    .padding(24)
    .into()
}
