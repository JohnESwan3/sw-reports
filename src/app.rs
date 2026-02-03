use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Element, Length, Task, Theme};

use crate::message::Message;
use crate::importing::{
    DuplicateEntry, ImportState, ImportStatus, ImportStep, PendingDuplicate, NewHireRecord,
};
use crate::screens::Page;
use crate::theme::{
    ACCENT, DRAWER_BG, DRAWER_ITEM_BG, DRAWER_TEXT_ACTIVE, DRAWER_TEXT_INACTIVE,
};
use lucide_icons::iced::{
    icon_chart_line, icon_house, icon_panel_left_close, icon_panel_left_open, icon_plus,
};
use std::collections::VecDeque;
use std::path::PathBuf;

pub struct App {
    theme: Theme,
    current_page: Page,
    sidebar_collapsed: bool,
    db_path: PathBuf,
    import_state: ImportState,
    import_queue: VecDeque<NewHireRecord>,
    pending_duplicates: VecDeque<DuplicateEntry>,
    decision_queue: VecDeque<(NewHireRecord, bool)>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let initial_page = Page::Home;
        let theme = Theme::Dark;
        let db_path = directories::ProjectDirs::from("com", "woodgrain", "sw-reports")
            .map(|dirs| dirs.data_dir().join("sw_reports.sqlite"))
            .unwrap_or_else(|| PathBuf::from("sw_reports.sqlite"));
        (
            Self {
                theme,
                current_page: initial_page,
                sidebar_collapsed: true,
                db_path,
                import_state: ImportState::new(),
                import_queue: VecDeque::new(),
                pending_duplicates: VecDeque::new(),
                decision_queue: VecDeque::new(),
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
            Message::Noop => Task::none(),
            Message::StartImport => {
                let file = rfd::FileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .pick_file();

                if let Some(path) = file {
                    return self.start_import_with_path(path);
                }

                Task::none()
            }
            Message::ImportPrepared(result) => match result {
                Ok(records) => {
                    let total = records.len();
                    self.import_queue = VecDeque::from(records);
                    self.pending_duplicates.clear();
                    self.decision_queue.clear();
                    self.import_state.start(total);
                    self.import_state.set_message("Processing records...".to_owned());

                    if total == 0 {
                        self.import_state.status = ImportStatus::Done;
                        self.import_state
                            .set_message("No data rows found.".to_owned());
                        return Task::none();
                    }

                    self.process_next_record()
                }
                Err(err) => {
                    self.import_state.set_error(err);
                    Task::none()
                }
            },
            Message::ProcessedRecord(result) => match result {
                Ok(step) => self.handle_import_step(step),
                Err(err) => {
                    self.import_state.set_error(err);
                    Task::none()
                }
            },
            Message::DecideDuplicate { number, overwrite } => {
                if let Some(index) = self
                    .pending_duplicates
                    .iter()
                    .position(|entry| entry.record.number == number)
                {
                    if let Some(record) = self.pending_duplicates.remove(index) {
                        self.import_state
                            .pending_duplicates
                            .retain(|pending| pending.number != number);
                        self.decision_queue.push_back((record.record, overwrite));
                        return self.process_next_decision();
                    }
                }

                Task::none()
            }
            Message::DecideAll { overwrite } => {
                while let Some(entry) = self.pending_duplicates.pop_front() {
                    self.decision_queue.push_back((entry.record, overwrite));
                }

                self.import_state.pending_duplicates.clear();
                self.process_next_decision()
            }
            Message::DecisionApplied(result) => match result {
                Ok(step) => self.handle_import_step(step),
                Err(err) => {
                    self.import_state.set_error(err);
                    Task::none()
                }
            },
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

        let pages = [Page::Import, Page::Home, Page::Reports]
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
            Page::Import => icon_plus(),
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
            Page::Import => crate::screens::import::view(&self.import_state),
            Page::Home => crate::screens::home::view(self.sidebar_collapsed),
            Page::Reports => crate::screens::reports::view(self.sidebar_collapsed),
        }
    }

    fn start_import_with_path(&mut self, path: std::path::PathBuf) -> Task<Message> {
        self.import_state.status = ImportStatus::Loading;
        self.import_state.set_message("Reading CSV...".to_owned());
        Task::perform(crate::importing::read_new_hire_csv(path), Message::ImportPrepared)
    }

    fn process_next_record(&mut self) -> Task<Message> {
        if let Some(record) = self.import_queue.pop_front() {
            let db_path = self.db_path.clone();
            return Task::perform(
                crate::importing::process_record(db_path, record),
                Message::ProcessedRecord,
            );
        }

        if self.pending_duplicates.is_empty() {
            self.import_state.status = ImportStatus::Done;
            self.import_state
                .set_message("Import complete.".to_owned());
        } else {
            self.import_state.status = ImportStatus::AwaitingDecision;
            self.import_state.set_message(format!(
                "{} duplicate record(s) need review.",
                self.pending_duplicates.len()
            ));
        }
        Task::none()
    }

    fn handle_import_step(&mut self, step: ImportStep) -> Task<Message> {
        match step {
            ImportStep::Inserted => {
                self.import_state.inserted += 1;
                self.process_next_record()
            }
            ImportStep::Updated => {
                self.import_state.updated += 1;
                self.process_next_decision()
            }
            ImportStep::SkippedUnchanged => {
                self.import_state.skipped += 1;
                self.process_next_record()
            }
            ImportStep::SkippedDecision => {
                self.import_state.skipped += 1;
                self.process_next_decision()
            }
            ImportStep::Duplicate(record) => {
                self.pending_duplicates.push_back(record.clone());
                self.import_state
                    .pending_duplicates
                    .push(record.summary.clone());
                self.process_next_record()
            }
        }
    }

    fn process_next_decision(&mut self) -> Task<Message> {
        if let Some((record, overwrite)) = self.decision_queue.pop_front() {
            self.import_state.status = ImportStatus::Importing;
            let db_path = self.db_path.clone();
            return Task::perform(
                crate::importing::apply_duplicate_decision(db_path, record, overwrite),
                Message::DecisionApplied,
            );
        }

        if self.pending_duplicates.is_empty() {
            self.import_state.status = ImportStatus::Done;
            self.import_state
                .set_message("Import complete.".to_owned());
        }

        Task::none()
    }
}
