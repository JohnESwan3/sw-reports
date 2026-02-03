use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{Point, Rectangle, Renderer, Theme, Vector};

use super::model::{BarSeries, InteractionConfig, InteractionState, LineChartConfig};

pub struct BarChart {
    cache: Cache,
    series: BarSeries,
    config: LineChartConfig,
    interaction: InteractionConfig,
}

impl BarChart {
    pub fn new(series: BarSeries) -> Self {
        Self {
            cache: Cache::new(),
            series,
            config: LineChartConfig::default(),
            interaction: InteractionConfig::default(),
        }
    }

    pub fn with_config(mut self, config: LineChartConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_interaction(mut self, interaction: InteractionConfig) -> Self {
        self.interaction = interaction;
        self
    }
}

impl canvas::Program<crate::message::Message> for BarChart {
    type State = InteractionState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<crate::message::Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.selection_start.is_some() {
                    state.selection_end = Some(*position);
                }
                if let Some(pan_start) = state.pan_start {
                    let delta = Vector::new(position.x - pan_start.x, position.y - pan_start.y);
                    state.pan = state.pan_origin + delta;
                }
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::CursorEntered)
            | canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !self.interaction.enable_zoom {
                    return None;
                }
                let scroll = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 60.0,
                };
                let factor = if scroll > 0.0 { 1.1 } else { 0.9 };
                state.zoom = (state.zoom * factor).clamp(0.5, 5.0);
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if !self.interaction.enable_selection {
                    return None;
                }
                state.selection_start = cursor.position_in(bounds);
                state.selection_end = None;
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if !self.interaction.enable_selection {
                    return None;
                }
                if state.selection_start.is_some() && state.selection_end.is_some() {
                    // keep selection for now
                } else {
                    state.selection_start = None;
                    state.selection_end = None;
                }
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if !self.interaction.enable_pan {
                    return None;
                }
                state.pan_start = cursor.position_in(bounds);
                state.pan_origin = state.pan;
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                if !self.interaction.enable_pan {
                    return None;
                }
                state.pan_start = None;
                Some(canvas::Action::request_redraw())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut geometries = Vec::new();
        if self.series.values.is_empty() {
            return geometries;
        }

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();
            let size = frame.size();
            let padding = self.config.padding;

            if size.width <= padding * 2.0 || size.height <= padding * 2.0 {
                return;
            }

            let left = padding;
            let top = padding;
            let right = size.width - padding;
            let bottom = size.height - padding;

            let x_axis = Path::line(Point::new(left, bottom), Point::new(right, bottom));
            let y_axis = Path::line(Point::new(left, bottom), Point::new(left, top));

            frame.stroke(
                &x_axis,
                Stroke::default()
                    .with_width(1.0)
                    .with_color(palette.background.weak.color),
            );
            frame.stroke(
                &y_axis,
                Stroke::default()
                    .with_width(1.0)
                    .with_color(palette.background.weak.color),
            );

            let max_value = self
                .series
                .values
                .iter()
                .map(|point| point.value)
                .fold(0.0_f32, f32::max)
                .max(1.0);

            let zoom = if state.zoom <= 0.0 { 1.0 } else { state.zoom };
            let bar_width = ((right - left) / self.series.values.len() as f32) * zoom;

            for (index, point) in self.series.values.iter().enumerate() {
                let x = left + index as f32 * bar_width + state.pan.x;
                let height = (point.value / max_value) * (bottom - top);
                let rect = Path::rectangle(
                    Point::new(x, bottom - height),
                    iced::Size::new(bar_width * 0.8, height),
                );
                frame.fill(&rect, self.series.color);

                if index % 2 == 0 {
                    frame.fill_text(Text {
                        content: point.label.clone(),
                        position: Point::new(x + bar_width * 0.4, bottom + 6.0),
                        color: palette.background.base.text,
                        size: 11.0.into(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        ..Text::default()
                    });
                }
            }
        });

        geometries.push(geometry);

        if let Some(cursor_pos) = cursor.position_in(bounds) {
            let mut overlay = Frame::new(renderer, bounds.size());
            let palette = theme.extended_palette();
            let padding = self.config.padding;
            let left = padding;
            let top = padding;
            let right = bounds.width - padding;
            let bottom = bounds.height - padding;

            if cursor_pos.x >= left
                && cursor_pos.x <= right
                && cursor_pos.y >= top
                && cursor_pos.y <= bottom
            {
                let zoom = if state.zoom <= 0.0 { 1.0 } else { state.zoom };
                let bar_width = ((right - left) / self.series.values.len() as f32) * zoom;
                let index = ((cursor_pos.x - left - state.pan.x) / bar_width).floor() as i32;

                if index >= 0 && (index as usize) < self.series.values.len() {
                    let point = &self.series.values[index as usize];
                    let label = format!("{}: {:.2}", point.label, point.value);
                    overlay.fill_text(Text {
                        content: label,
                        position: Point::new(cursor_pos.x + 8.0, cursor_pos.y - 8.0),
                        color: palette.background.base.text,
                        size: 12.0.into(),
                        ..Text::default()
                    });
                }
            }

            if let (Some(start), Some(end)) = (state.selection_start, state.selection_end) {
                let selection_left = start.x.min(end.x);
                let selection_right = start.x.max(end.x);
                let selection_top = start.y.min(end.y);
                let selection_bottom = start.y.max(end.y);
                let rect = Path::rectangle(
                    Point::new(selection_left, selection_top),
                    iced::Size::new(
                        (selection_right - selection_left).max(1.0),
                        (selection_bottom - selection_top).max(1.0),
                    ),
                );
                overlay.stroke(
                    &rect,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(palette.primary.strong.color),
                );
            }

            geometries.push(overlay.into_geometry());
        }

        geometries
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.position_in(bounds).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
