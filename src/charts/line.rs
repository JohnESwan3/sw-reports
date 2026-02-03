use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{Point, Rectangle, Renderer, Theme, Vector};

use super::model::{InteractionConfig, InteractionState, LineChartConfig, LineSeries};

pub struct LineChart {
    cache: Cache,
    series: Vec<LineSeries>,
    x_range: Option<(f32, f32)>,
    y_range: Option<(f32, f32)>,
    config: LineChartConfig,
    interaction: InteractionConfig,
}

impl LineChart {
    pub fn new(series: Vec<LineSeries>) -> Self {
        Self {
            cache: Cache::new(),
            series,
            x_range: None,
            y_range: None,
            config: LineChartConfig::default(),
            interaction: InteractionConfig::default(),
        }
    }

    pub fn with_x_range(mut self, range: (f32, f32)) -> Self {
        self.x_range = Some(range);
        self
    }

    pub fn with_y_range(mut self, range: (f32, f32)) -> Self {
        self.y_range = Some(range);
        self
    }

    pub fn with_config(mut self, config: LineChartConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_interaction(mut self, interaction: InteractionConfig) -> Self {
        self.interaction = interaction;
        self
    }

    fn data_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for series in &self.series {
            for (x, y) in &series.points {
                min_x = min_x.min(*x);
                max_x = max_x.max(*x);
                min_y = min_y.min(*y);
                max_y = max_y.max(*y);
            }
        }

        if min_x.is_finite() && min_y.is_finite() {
            Some((min_x, max_x, min_y, max_y))
        } else {
            None
        }
    }

    fn ranges(&self) -> Option<(f32, f32, f32, f32)> {
        let (min_x, max_x, min_y, max_y) = self.data_bounds()?;
        let (x_min, x_max) = self.x_range.unwrap_or((min_x, max_x));
        let (y_min, y_max) = self.y_range.unwrap_or((min_y, max_y));
        Some((x_min, x_max, y_min, y_max))
    }
}

impl canvas::Program<crate::message::Message> for LineChart {
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
                let factor = if scroll > 0.0 { 0.9 } else { 1.1 };
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
                if let Some(pos) = cursor.position_in(bounds) {
                    state.pan_start = Some(pos);
                    state.pan_origin = state.pan;
                }
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
        let Some((x_min, x_max, y_min, y_max)) = self.ranges() else {
            return geometries;
        };

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();
            let size = frame.size();
            let padding = self.config.padding;
            let zoom = if state.zoom <= 0.0 { 1.0 } else { state.zoom };
            let pan = state.pan;

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

            let grid_lines = self.config.grid_lines.max(1);
            for i in 0..=grid_lines {
                let t = i as f32 / grid_lines as f32;
                let y = bottom - t * (bottom - top);
                let line = Path::line(Point::new(left, y), Point::new(right, y));
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(palette.background.weak.color),
                );

                let value = y_min + t * (y_max - y_min);
                frame.fill_text(Text {
                    content: format!("{value:.2}"),
                    position: Point::new(left - 8.0, y - 6.0),
                    color: palette.background.base.text,
                    size: 11.0.into(),
                    align_x: iced::alignment::Horizontal::Right.into(),
                    ..Text::default()
                });
            }

            for i in 0..=grid_lines {
                let t = i as f32 / grid_lines as f32;
                let x = left + t * (right - left);
                let line = Path::line(Point::new(x, top), Point::new(x, bottom));
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(palette.background.weak.color),
                );

                let value = x_min + t * (x_max - x_min);
                frame.fill_text(Text {
                    content: format!("{value:.2}"),
                    position: Point::new(x, bottom + 8.0),
                    color: palette.background.base.text,
                    size: 11.0.into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    ..Text::default()
                });
            }

            let x_range = ((x_max - x_min).max(1.0)) / zoom;
            let y_range = ((y_max - y_min).max(1.0)) / zoom;
            let x_offset = pan.x;
            let y_offset = pan.y;

            for series in &self.series {
                if series.points.len() < 2 {
                    continue;
                }

                let path = Path::new(|builder| {
                    for (index, (x, y)) in series.points.iter().enumerate() {
                        let x = left + ((x - x_min) / x_range) * (right - left) + x_offset;
                        let y = bottom - ((y - y_min) / y_range) * (bottom - top) + y_offset;

                        if index == 0 {
                            builder.move_to(Point::new(x, y));
                        } else {
                            builder.line_to(Point::new(x, y));
                        }
                    }
                });

                frame.stroke(
                    &path,
                    Stroke::default().with_width(2.0).with_color(series.color),
                );
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
            let zoom = if state.zoom <= 0.0 { 1.0 } else { state.zoom };
            let pan = state.pan;

            if cursor_pos.x >= left
                && cursor_pos.x <= right
                && cursor_pos.y >= top
                && cursor_pos.y <= bottom
            {
                let mut nearest = None;
                let x_range = ((x_max - x_min).max(1.0)) / zoom;
                let y_range = ((y_max - y_min).max(1.0)) / zoom;

                for series in &self.series {
                    for (x, y) in &series.points {
                        let screen_x = left + ((x - x_min) / x_range) * (right - left) + pan.x;
                        let screen_y = bottom - ((y - y_min) / y_range) * (bottom - top) + pan.y;
                        let dx = screen_x - cursor_pos.x;
                        let dy = screen_y - cursor_pos.y;
                        let distance = dx * dx + dy * dy;

                        if nearest
                            .as_ref()
                            .map(|(_, d)| distance < *d)
                            .unwrap_or(true)
                        {
                            nearest = Some((
                                (screen_x, screen_y, *x, *y, series.name.as_str(), series.color),
                                distance,
                            ));
                        }
                    }
                }

                if let Some(((sx, sy, x, y, name, color), _)) = nearest {
                    let v_line = Path::line(Point::new(sx, top), Point::new(sx, bottom));
                    let h_line = Path::line(Point::new(left, sy), Point::new(right, sy));
                    overlay.stroke(
                        &v_line,
                        Stroke::default()
                            .with_width(1.0)
                            .with_color(palette.background.weak.color),
                    );
                    overlay.stroke(
                        &h_line,
                        Stroke::default()
                            .with_width(1.0)
                            .with_color(palette.background.weak.color),
                    );

                    let point = Path::circle(Point::new(sx, sy), 3.5);
                    overlay.fill(&point, color);

                    let label = format!("{name}: x={x:.2}, y={y:.2}");
                    let tooltip_padding = 6.0;
                    let tooltip_width = label.len() as f32 * 7.0 + tooltip_padding * 2.0;
                    let tooltip_height = 20.0;
                    let mut tooltip_x = sx + 10.0;
                    let mut tooltip_y = sy - tooltip_height - 10.0;

                    if tooltip_x + tooltip_width > right {
                        tooltip_x = sx - tooltip_width - 10.0;
                    }
                    if tooltip_y < top {
                        tooltip_y = sy + 10.0;
                    }

                    let rect = Path::rectangle(
                        Point::new(tooltip_x, tooltip_y),
                        iced::Size::new(tooltip_width, tooltip_height),
                    );
                    overlay.fill(&rect, palette.background.strong.color);
                    overlay.stroke(
                        &rect,
                        Stroke::default()
                            .with_width(1.0)
                            .with_color(palette.background.weak.color),
                    );
                    overlay.fill_text(Text {
                        content: label,
                        position: Point::new(tooltip_x + tooltip_padding, tooltip_y + 4.0),
                        color: palette.background.strong.text,
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
