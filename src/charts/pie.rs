use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Text};
use iced::{Point, Radians, Rectangle, Renderer, Theme, Vector};

use super::model::{InteractionConfig, InteractionState, PieSlice};

#[derive(Default)]
pub struct PieState {
    interaction: InteractionState,
    selected_index: Option<usize>,
}

pub struct PieChart {
    cache: Cache,
    slices: Vec<PieSlice>,
    interaction: InteractionConfig,
}

impl PieChart {
    pub fn new(slices: Vec<PieSlice>) -> Self {
        Self {
            cache: Cache::new(),
            slices,
            interaction: InteractionConfig::default(),
        }
    }

    pub fn with_interaction(mut self, interaction: InteractionConfig) -> Self {
        self.interaction = interaction;
        self
    }
}

impl canvas::Program<crate::message::Message> for PieChart {
    type State = PieState;

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
                state.interaction.zoom = (state.interaction.zoom * factor).clamp(0.5, 5.0);
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(pan_start) = state.interaction.pan_start {
                    let delta =
                        Vector::new(position.x - pan_start.x, position.y - pan_start.y);
                    state.interaction.pan = state.interaction.pan_origin + delta;
                }
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if !self.interaction.enable_pan {
                    return None;
                }
                state.interaction.pan_start = cursor.position_in(bounds);
                state.interaction.pan_origin = state.interaction.pan;
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                if !self.interaction.enable_pan {
                    return None;
                }
                state.interaction.pan_start = None;
                Some(canvas::Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if !self.interaction.enable_selection {
                    return None;
                }
                if let Some(pos) = cursor.position_in(bounds) {
                    state.selected_index = hit_test_slice(&self.slices, bounds, pos, &state.interaction);
                }
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
        if self.slices.is_empty() {
            return geometries;
        }

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let size = frame.size();
            let center = Point::new(size.width / 2.0, size.height / 2.0)
                + state.interaction.pan;
            let radius = (size.width.min(size.height) * 0.35)
                * if state.interaction.zoom <= 0.0 {
                    1.0
                } else {
                    state.interaction.zoom
                };

            let total: f32 = self.slices.iter().map(|slice| slice.value).sum();
            if total <= 0.0 {
                return;
            }

            let mut start = -std::f32::consts::FRAC_PI_2;
            for (index, slice) in self.slices.iter().enumerate() {
                let sweep = (slice.value / total) * std::f32::consts::TAU;
                let end = start + sweep;

                let path = Path::new(|builder| {
                    builder.move_to(center);
                    builder.arc(canvas::path::Arc {
                        center,
                        radius,
                        start_angle: Radians(start),
                        end_angle: Radians(end),
                    });
                    builder.close();
                });

                let color = if state.selected_index == Some(index) {
                    brighten(slice.color, 1.15)
                } else {
                    slice.color
                };

                frame.fill(&path, color);
                start = end;
            }
        });

        geometries.push(geometry);

        if let Some(cursor_pos) = cursor.position_in(bounds) {
            let mut overlay = Frame::new(renderer, bounds.size());
            let palette = theme.extended_palette();

            if let Some(index) =
                hit_test_slice(&self.slices, bounds, cursor_pos, &state.interaction)
            {
                let slice = &self.slices[index];
                overlay.fill_text(Text {
                    content: format!("{}: {:.2}", slice.label, slice.value),
                    position: Point::new(cursor_pos.x + 8.0, cursor_pos.y - 8.0),
                    color: palette.background.base.text,
                    size: 12.0.into(),
                    ..Text::default()
                });
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

fn hit_test_slice(
    slices: &[PieSlice],
    bounds: Rectangle,
    cursor_pos: Point,
    interaction: &InteractionState,
) -> Option<usize> {
    if slices.is_empty() {
        return None;
    }

    let size = bounds.size();
    let center = Point::new(size.width / 2.0, size.height / 2.0) + interaction.pan;
    let radius = (size.width.min(size.height) * 0.35)
        * if interaction.zoom <= 0.0 {
            1.0
        } else {
            interaction.zoom
        };

    let dx = cursor_pos.x - center.x;
    let dy = cursor_pos.y - center.y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance > radius {
        return None;
    }

    let mut angle = dy.atan2(dx);
    if angle < -std::f32::consts::FRAC_PI_2 {
        angle += std::f32::consts::TAU;
    }

    let total: f32 = slices.iter().map(|slice| slice.value).sum();
    if total <= 0.0 {
        return None;
    }

    let mut start = -std::f32::consts::FRAC_PI_2;
    for (index, slice) in slices.iter().enumerate() {
        let sweep = (slice.value / total) * std::f32::consts::TAU;
        let end = start + sweep;
        if angle >= start && angle <= end {
            return Some(index);
        }
        start = end;
    }

    None
}

fn brighten(color: iced::Color, factor: f32) -> iced::Color {
    iced::Color {
        r: (color.r * factor).min(1.0),
        g: (color.g * factor).min(1.0),
        b: (color.b * factor).min(1.0),
        a: color.a,
    }
}
