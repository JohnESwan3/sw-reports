use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{Point, Radians, Rectangle, Renderer, Theme, Vector};

use super::model::{CircleMetric, InteractionConfig, InteractionState};

pub struct CircleChart {
    cache: Cache,
    metric: CircleMetric,
    interaction: InteractionConfig,
}

impl CircleChart {
    pub fn new(metric: CircleMetric) -> Self {
        Self {
            cache: Cache::new(),
            metric,
            interaction: InteractionConfig::default(),
        }
    }

    pub fn with_interaction(mut self, interaction: InteractionConfig) -> Self {
        self.interaction = interaction;
        self
    }
}

impl canvas::Program<crate::message::Message> for CircleChart {
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

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();
            let size = frame.size();
            let center = Point::new(size.width / 2.0, size.height / 2.0) + state.pan;
            let radius = (size.width.min(size.height) * 0.35)
                * if state.zoom <= 0.0 { 1.0 } else { state.zoom };

            let background = Path::circle(center, radius);
            frame.stroke(
                &background,
                Stroke::default()
                    .with_width(12.0)
                    .with_color(palette.background.weak.color),
            );

            let ratio = (self.metric.value / self.metric.max).clamp(0.0, 1.0);
            let sweep = ratio * std::f32::consts::TAU;
            let start = -std::f32::consts::FRAC_PI_2;
            let end = start + sweep;

            let arc = Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle: Radians(start),
                    end_angle: Radians(end),
                });
            });
            frame.stroke(
                &arc,
                Stroke::default()
                    .with_width(12.0)
                    .with_color(self.metric.color),
            );

            frame.fill_text(Text {
                content: format!("{:.0}%", ratio * 100.0),
                position: center,
                color: palette.background.base.text,
                size: 20.0.into(),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center.into(),
                ..Text::default()
            });

            frame.fill_text(Text {
                content: self.metric.label.clone(),
                position: Point::new(center.x, center.y + 22.0),
                color: palette.background.base.text,
                size: 12.0.into(),
                align_x: iced::alignment::Horizontal::Center.into(),
                ..Text::default()
            });
        });

        geometries.push(geometry);

        if let Some(cursor_pos) = cursor.position_in(bounds) {
            let mut overlay = Frame::new(renderer, bounds.size());
            let palette = theme.extended_palette();
            overlay.fill_text(Text {
                content: format!(
                    "{}: {:.2} / {:.2}",
                    self.metric.label, self.metric.value, self.metric.max
                ),
                position: Point::new(cursor_pos.x + 8.0, cursor_pos.y - 8.0),
                color: palette.background.base.text,
                size: 12.0.into(),
                ..Text::default()
            });
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
