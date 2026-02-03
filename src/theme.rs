use iced::{widget::button, Background, Color, Theme};

pub const ACCENT: Color = Color::from_rgb8(0x06, 0x5f, 0x46);
pub const DRAWER_BG: Color = Color::from_rgb8(0x0b, 0x14, 0x12);
pub const DRAWER_ITEM_BG: Color = Color::from_rgb8(0x0f, 0x1f, 0x1a);
pub const DRAWER_TEXT_ACTIVE: Color = Color::from_rgb8(0xe6, 0xf4, 0xf1);
pub const DRAWER_TEXT_INACTIVE: Color = Color::from_rgb8(0xa5, 0xb3, 0xad);
pub const TEXT_ON_ACCENT: Color = Color::from_rgb8(0xe9, 0xf7, 0xf3);

pub fn accent_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let mut background = ACCENT;

    if matches!(status, button::Status::Hovered) {
        background.a = 0.85;
    }

    if matches!(status, button::Status::Pressed) {
        background.a = 0.7;
    }

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT_ON_ACCENT,
        ..Default::default()
    }
}
