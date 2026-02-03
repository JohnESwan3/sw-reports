use std::path::PathBuf;

use iced::Color;

use crate::charts::{LineChart, LineChartConfig, LineSeries};
use crate::data::lead_time;

pub struct ItLeadTimeReport;

impl ItLeadTimeReport {
    pub fn title() -> &'static str {
        "IT Lead Time"
    }

    pub fn subtitle() -> &'static str {
        "Elapsed hours by ticket number"
    }

    pub async fn load(db_path: PathBuf) -> Result<Vec<(f32, f32)>, String> {
        lead_time::load_series(db_path).await
    }

    pub fn chart(points: &[(f32, f32)]) -> LineChart {
        let series = vec![LineSeries::new(
            "IT Lead Time (Elapsed)",
            Color::from_rgb(0.35, 0.62, 0.96),
            points.to_vec(),
        )];

        LineChart::new(series).with_config(LineChartConfig {
            padding: 40.0,
            grid_lines: 5,
        })
    }
}
