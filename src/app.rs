use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Element, Length, Task, Theme};

use crate::message::Message;
use crate::screens::Page;
use crate::theme::{
    ACCENT, DRAWER_BG, DRAWER_ITEM_BG, DRAWER_TEXT_ACTIVE, DRAWER_TEXT_INACTIVE,
};
use lucide_icons::iced::{
    icon_chart_line, icon_house, icon_panel_left_close, icon_panel_left_open,
};

pub struct App {
    theme: Theme,
    current_page: Page,
    sidebar_collapsed: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let initial_page = Page::Home;
        let theme = Theme::Dark;
        (
            Self {
                theme,
                current_page: initial_page,
                sidebar_collapsed: true,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                Task::none()
            }
            Message::Navigate(page) => {
                self.current_page = page;
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let sidebar = self.sidebar_view();
        let content = self.content_view();

        row![sidebar, content].height(Length::Fill).into()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn sidebar_view<'a>(&'a self) -> Element<'a, Message> {
        let toggle_icon = if self.sidebar_collapsed {
            icon_panel_left_open()
        } else {
            icon_panel_left_close()
        };

        let toggle = button(toggle_icon.size(18))
            .on_press(Message::ToggleSidebar)
            .style(|_theme, status| {
                let mut background = ACCENT;
                if matches!(status, button::Status::Hovered) {
                    background.a = 0.85;
                }
                if matches!(status, button::Status::Pressed) {
                    background.a = 0.7;
                }

                button::Style {
                    background: Some(Background::Color(background)),
                    text_color: DRAWER_TEXT_ACTIVE,
                    ..Default::default()
                }
            });

        let pages = [Page::Home, Page::Reports]
            .into_iter()
            .map(|page| self.sidebar_button(page));

        let content = column![toggle, Space::new().height(Length::Fixed(12.0))]
            .push(column(pages).spacing(6))
            .spacing(12)
            .padding(12)
            .width(if self.sidebar_collapsed {
                Length::Fixed(64.0)
            } else {
                Length::Fixed(220.0)
            })
            .height(Length::Fill);

        container(content)
            .style(|_| iced::widget::container::background(DRAWER_BG))
            .into()
    }

    fn sidebar_button<'a>(&'a self, page: Page) -> Element<'a, Message> {
        let selected = self.current_page == page;
        let label = page.label();
        let icon = match page {
            Page::Home => icon_house(),
            Page::Reports => icon_chart_line(),
        }
        .size(18)
        .style(move |_| iced::widget::text::Style {
            color: Some(if selected {
                DRAWER_TEXT_ACTIVE
            } else {
                DRAWER_TEXT_INACTIVE
            }),
        });

        let label_text = text(label).style(move |_| iced::widget::text::Style {
            color: Some(if selected {
                DRAWER_TEXT_ACTIVE
            } else {
                DRAWER_TEXT_INACTIVE
            }),
        });

        let row_content = if self.sidebar_collapsed {
            row![
                Space::new().width(Length::Fill),
                icon,
                Space::new().width(Length::Fill)
            ]
            .align_y(Alignment::Center)
        } else {
            row![icon, label_text]
                .spacing(12)
                .align_y(Alignment::Center)
        };

        button(row_content)
            .on_press(Message::Navigate(page))
            .width(Length::Fill)
            .style(move |_, status| {
                let background = if selected {
                    ACCENT
                } else {
                    DRAWER_ITEM_BG
                };

                let mut color = background;
                if matches!(status, button::Status::Hovered) {
                    color.a = 0.85;
                }
                if matches!(status, button::Status::Pressed) {
                    color.a = 0.7;
                }

                button::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                }
            })
            .padding(8)
            .into()
    }

    fn content_view<'a>(&'a self) -> Element<'a, Message> {
        match self.current_page {
            Page::Home => crate::screens::home::view(self.sidebar_collapsed),
            Page::Reports => crate::screens::reports::view(self.sidebar_collapsed),
        }
    }
}
