use std::path::PathBuf;

use iced::Color;

use crate::charts::{BarChart, BarPoint, BarSeries, LineChartConfig};
use crate::data::state_counts;

pub struct StateCountsBarReport;

impl StateCountsBarReport {
    pub fn title() -> &'static str {
        "Requests by State"
    }

    pub fn subtitle() -> &'static str {
        "Top 10 states by total records"
    }

    pub async fn load(db_path: PathBuf) -> Result<Vec<(String, f32)>, String> {
        state_counts::load_series(db_path).await
    }

    pub fn chart(points: &[(String, f32)]) -> BarChart {
        let values = points
            .iter()
            .map(|(label, value)| BarPoint {
                label: label.clone(),
                value: *value,
            })
            .collect();

        BarChart::new(BarSeries {
            name: "Requests".to_string(),
            color: Color::from_rgb(0.35, 0.62, 0.96),
            values,
        })
        .with_config(LineChartConfig {
            padding: 40.0,
            grid_lines: 4,
        })
    }
}
