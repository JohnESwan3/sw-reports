use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{Point, Rectangle, Renderer, Theme, Vector};

use super::model::{InteractionConfig, InteractionState, RadarAxes, RadarSeries};

pub struct RadarChart {
    cache: Cache,
    axes: RadarAxes,
    series: Vec<RadarSeries>,
    interaction: InteractionConfig,
}

impl RadarChart {
    pub fn new(axes: RadarAxes, series: Vec<RadarSeries>) -> Self {
        Self {
            cache: Cache::new(),
            axes,
            series,
            interaction: InteractionConfig::default(),
        }
    }

    pub fn with_interaction(mut self, interaction: InteractionConfig) -> Self {
        self.interaction = interaction;
        self
    }
}

impl canvas::Program<crate::message::Message> for RadarChart {
    type State = InteractionState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<crate::message::Message>> {
        match event {
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
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(pan_start) = state.pan_start {
                    let delta = Vector::new(position.x - pan_start.x, position.y - pan_start.y);
                    state.pan = state.pan_origin + delta;
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
        if self.axes.labels.is_empty() || self.series.is_empty() {
            return geometries;
        }

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();
            let size = frame.size();
            let center = Point::new(size.width / 2.0, size.height / 2.0) + state.pan;
            let radius = (size.width.min(size.height) * 0.35)
                * if state.zoom <= 0.0 { 1.0 } else { state.zoom };
            let axis_count = self.axes.labels.len();
            let step = std::f32::consts::TAU / axis_count as f32;

            for ring in 1..=4 {
                let r = radius * (ring as f32 / 4.0);
                let circle = Path::circle(center, r);
                frame.stroke(
                    &circle,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(palette.background.weak.color),
                );
            }

            for i in 0..axis_count {
                let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * step;
                let x = center.x + radius * angle.cos();
                let y = center.y + radius * angle.sin();
                let axis = Path::line(center, Point::new(x, y));
                frame.stroke(
                    &axis,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(palette.background.weak.color),
                );

                let label = &self.axes.labels[i];
                frame.fill_text(Text {
                    content: label.clone(),
                    position: Point::new(x, y),
                    color: palette.background.base.text,
                    size: 11.0.into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center.into(),
                    ..Text::default()
                });
            }

            for series in &self.series {
                let mut points = Vec::new();
                for (i, value) in series.values.iter().enumerate() {
                    let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * step;
                    let magnitude = (value / self.axes.max_value).clamp(0.0, 1.0);
                    let x = center.x + radius * magnitude * angle.cos();
                    let y = center.y + radius * magnitude * angle.sin();
                    points.push(Point::new(x, y));
                }

                let polygon = Path::new(|builder| {
                    if let Some(first) = points.first() {
                        builder.move_to(*first);
                        for point in points.iter().skip(1) {
                            builder.line_to(*point);
                        }
                        builder.close();
                    }
                });
                frame.stroke(
                    &polygon,
                    Stroke::default().with_width(2.0).with_color(series.color),
                );
            }
        });

        geometries.push(geometry);

        if let Some(cursor_pos) = cursor.position_in(bounds) {
            let mut overlay = Frame::new(renderer, bounds.size());
            let palette = theme.extended_palette();

            let size = bounds.size();
            let center = Point::new(size.width / 2.0, size.height / 2.0) + state.pan;
            let dx = cursor_pos.x - center.x;
            let dy = cursor_pos.y - center.y;
            let angle = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
            let normalized = if angle < 0.0 {
                angle + std::f32::consts::TAU
            } else {
                angle
            };
            let axis_count = self.axes.labels.len();
            let step = std::f32::consts::TAU / axis_count as f32;
            let axis_index = (normalized / step).round() as usize % axis_count;

            if let Some(series) = self.series.first() {
                if axis_index < series.values.len() {
                    overlay.fill_text(Text {
                        content: format!(
                            "{}: {:.2}",
                            self.axes.labels[axis_index], series.values[axis_index]
                        ),
                        position: Point::new(cursor_pos.x + 8.0, cursor_pos.y - 8.0),
                        color: palette.background.base.text,
                        size: 12.0.into(),
                        ..Text::default()
                    });
                }
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
