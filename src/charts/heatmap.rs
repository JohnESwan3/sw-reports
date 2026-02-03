use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Theme, Vector};

use super::model::{HeatmapGrid, InteractionConfig, InteractionState, LineChartConfig};

pub struct HeatmapChart {
    cache: Cache,
    grid: HeatmapGrid,
    config: LineChartConfig,
    interaction: InteractionConfig,
}

impl HeatmapChart {
    pub fn new(grid: HeatmapGrid) -> Self {
        Self {
            cache: Cache::new(),
            grid,
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

impl canvas::Program<crate::message::Message> for HeatmapChart {
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
                state.zoom = (state.zoom * factor).clamp(0.5, 4.0);
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
        if self.grid.x_labels.is_empty() || self.grid.y_labels.is_empty() {
            return geometries;
        }

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();
            let size = frame.size();
            let padding = self.config.padding;
            let zoom = if state.zoom <= 0.0 { 1.0 } else { state.zoom };

            if size.width <= padding * 2.0 || size.height <= padding * 2.0 {
                return;
            }

            let left = padding;
            let top = padding;
            let right = size.width - padding;
            let bottom = size.height - padding;

            let cols = self.grid.x_labels.len().max(1) as f32;
            let rows = self.grid.y_labels.len().max(1) as f32;
            let cell_width = ((right - left) / cols) * zoom;
            let cell_height = ((bottom - top) / rows) * zoom;

            let mut min = f32::INFINITY;
            let mut max = f32::NEG_INFINITY;
            for row in &self.grid.values {
                for value in row {
                    min = min.min(*value);
                    max = max.max(*value);
                }
            }
            let range = (max - min).max(1.0);

            for (y, row) in self.grid.values.iter().enumerate() {
                for (x, value) in row.iter().enumerate() {
                    let px = left + x as f32 * cell_width + state.pan.x;
                    let py = top + y as f32 * cell_height + state.pan.y;
                    let intensity = (*value - min) / range;
                    let color = lerp_color(
                        palette.background.weak.color,
                        palette.primary.strong.color,
                        intensity,
                    );
                    let rect = Path::rectangle(
                        Point::new(px, py),
                        iced::Size::new(cell_width.max(1.0), cell_height.max(1.0)),
                    );
                    frame.fill(&rect, color);
                }
            }

            for (x, label) in self.grid.x_labels.iter().enumerate() {
                let px = left + x as f32 * cell_width + state.pan.x + cell_width / 2.0;
                frame.fill_text(Text {
                    content: label.clone(),
                    position: Point::new(px, bottom + 8.0),
                    color: palette.background.base.text,
                    size: 11.0.into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    ..Text::default()
                });
            }

            for (y, label) in self.grid.y_labels.iter().enumerate() {
                let py = top + y as f32 * cell_height + state.pan.y + cell_height / 2.0;
                frame.fill_text(Text {
                    content: label.clone(),
                    position: Point::new(left - 8.0, py),
                    color: palette.background.base.text,
                    size: 11.0.into(),
                    align_x: iced::alignment::Horizontal::Right.into(),
                    align_y: iced::alignment::Vertical::Center.into(),
                    ..Text::default()
                });
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
                let cols = self.grid.x_labels.len().max(1) as f32;
                let rows = self.grid.y_labels.len().max(1) as f32;
                let cell_width = ((right - left) / cols) * zoom;
                let cell_height = ((bottom - top) / rows) * zoom;

                let x_index =
                    ((cursor_pos.x - left - state.pan.x) / cell_width).floor() as i32;
                let y_index =
                    ((cursor_pos.y - top - state.pan.y) / cell_height).floor() as i32;

                if x_index >= 0
                    && y_index >= 0
                    && (x_index as usize) < self.grid.x_labels.len()
                    && (y_index as usize) < self.grid.y_labels.len()
                {
                    let value = self.grid.values[y_index as usize][x_index as usize];
                    overlay.fill_text(Text {
                        content: format!(
                            "{} / {}: {:.2}",
                            self.grid.y_labels[y_index as usize],
                            self.grid.x_labels[x_index as usize],
                            value
                        ),
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
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}
