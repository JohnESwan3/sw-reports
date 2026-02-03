use std::path::PathBuf;

use iced::Color;

use crate::charts::{CircleChart, CircleMetric};
use crate::data::sla_breach_rate;

pub struct SlaBreachCircleReport;

impl SlaBreachCircleReport {
    pub fn title() -> &'static str {
        "SLA Breach Rate"
    }

    pub fn subtitle() -> &'static str {
        "Share of records with SLA breaches"
    }

    pub async fn load(db_path: PathBuf) -> Result<(f32, f32), String> {
        sla_breach_rate::load_rate(db_path).await
    }

    pub fn chart(breaches: f32, total: f32) -> CircleChart {
        CircleChart::new(CircleMetric {
            label: "Breaches".to_string(),
            value: breaches,
            max: total,
            color: Color::from_rgb(0.89, 0.40, 0.40),
        })
    }
}
