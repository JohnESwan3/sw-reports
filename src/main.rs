mod app;
mod charts;
mod data;
mod importing;
mod message;
mod reports;
mod screens;
mod theme;

use app::App;
use iced::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;

fn main() -> iced::Result{
    iced::application(App::new, App::update, App::view)
        .theme(App::theme)
        .settings(Settings {
            fonts: vec![LUCIDE_FONT_BYTES.into()],
            ..Default::default()
        })
        .window_size((1024.0, 768.0))
        .run()
}
