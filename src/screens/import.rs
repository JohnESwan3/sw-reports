use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Background, Element, Length, Padding};
use lucide_icons::iced::icon_file_plus;

use crate::importing::{ImportState, ImportStatus};
use crate::message::Message;
use crate::theme::{accent_button_style, DRAWER_BG, DRAWER_ITEM_BG, DRAWER_TEXT_ACTIVE, DRAWER_TEXT_INACTIVE};

pub fn view<'a>(import_state: &ImportState) -> Element<'a, Message> {
    let status_text = match import_state.status {
        ImportStatus::Idle => "No import in progress.".to_owned(),
        ImportStatus::Loading => "Reading CSV...".to_owned(),
        ImportStatus::Importing => "Importing records...".to_owned(),
        ImportStatus::AwaitingDecision => "Duplicate found. Choose overwrite or skip.".to_owned(),
        ImportStatus::Done => "Import complete.".to_owned(),
        ImportStatus::Error => "Import failed.".to_owned(),
    };

    let counts = format!(
        "Total: {} | Inserted: {} | Updated: {} | Skipped: {}",
        import_state.total,
        import_state.inserted,
        import_state.updated,
        import_state.skipped
    );

    let duplicates_table = if import_state.pending_duplicates.is_empty() {
        None
    } else {
        let header = container(
            row![
                text("Number").size(14),
                text("Title").size(14),
                text("Created At").size(14),
                text("Changes").size(14).width(Length::Fill),
                Space::new().width(Length::Fixed(16.0)),
            ]
            .spacing(16),
        )
        .padding(Padding::new(0.0).right(16.0));

        let rows = import_state.pending_duplicates.iter().map(|pending| {
            let title = pending.title.clone().unwrap_or_else(|| "Unknown Title".to_owned());
            let created_at = pending
                .created_at
                .clone()
                .unwrap_or_else(|| "Unknown timestamp".to_owned());

            let changes = if pending.changes.is_empty() {
                "no changes".to_owned()
            } else {
                pending.changes.join(", ")
            };

            container(
                row![
                    text(pending.number).size(14),
                    text(title).size(14).width(Length::Fill),
                    text(created_at).size(14),
                    text(changes).size(13).width(Length::Fill),
                    row![
                        button("Overwrite")
                            .style(accent_button_style)
                            .on_press(Message::DecideDuplicate {
                                number: pending.number,
                                overwrite: true
                            }),
                        button("Skip")
                            .style(secondary_button_style)
                            .on_press(Message::DecideDuplicate {
                                number: pending.number,
                                overwrite: false
                            })
                    ]
                    .spacing(8),
                    Space::new().width(Length::Fixed(16.0)),
                ]
                .spacing(16),
            )
            .padding(Padding::new(0.0).right(16.0))
            .into()
        });

        let table = column![header, Space::new().height(Length::Fixed(8.0))]
            .push(column(rows).spacing(6))
            .spacing(8);

        Some(scrollable(table).height(Length::Fixed(260.0)))
    };

    let import_button = button(row![icon_file_plus().size(16), text("Upload CSV")].spacing(8))
        .style(accent_button_style)
        .on_press(Message::StartImport);

    let info_panel = container(
        column![
            text("New Hire Reports").size(22),
            text("Import the SolarWinds Service Desk CSV export to update metrics.")
                .size(14)
                .style(|_| text::Style {
                    color: Some(DRAWER_TEXT_INACTIVE),
                }),
            Space::new().height(Length::Fixed(8.0)),
            import_button,
            Space::new().height(Length::Fixed(8.0)),
            text(status_text).size(14),
            text(counts).size(14),
            import_state
                .message
                .as_ref()
                .map(|message| text(message.clone()).size(14))
                .unwrap_or_else(|| text("")),
            duplicates_table
                .map(|table| {
                    column![
                        text(format!(
                            "{} duplicate record(s)",
                            import_state.pending_duplicates.len()
                        ))
                        .size(16),
                        row![
                            button("Overwrite All")
                                .style(accent_button_style)
                                .on_press(Message::DecideAll { overwrite: true }),
                            button("Skip All")
                                .style(secondary_button_style)
                                .on_press(Message::DecideAll { overwrite: false })
                        ]
                        .spacing(12),
                        table
                    ]
                    .spacing(12)
                })
                .unwrap_or_else(|| column![].into())
        ]
        .spacing(12),
    )
    .padding(24)
    .width(Length::Fill)
    .max_width(1100)
    .style(|_| container::background(Background::Color(DRAWER_BG)));

    container(info_panel)
        .padding(24)
        .center_x(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn secondary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let mut background = DRAWER_ITEM_BG;

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
}
