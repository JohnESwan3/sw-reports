use std::path::PathBuf;

use iced::Color;

use crate::charts::{RadarAxes, RadarChart, RadarSeries};
use crate::data::radar_metrics;

pub struct LeadTimeRadarReport;

impl LeadTimeRadarReport {
    pub fn title() -> &'static str {
        "Lead Time Overview"
    }

    pub fn subtitle() -> &'static str {
        "Operational metrics snapshot"
    }

    pub async fn load(db_path: PathBuf) -> Result<Vec<(String, f32)>, String> {
        radar_metrics::load_metrics(db_path).await
    }

    pub fn chart(metrics: &[(String, f32)]) -> RadarChart {
        let labels = metrics.iter().map(|(label, _)| label.clone()).collect();
        let values: Vec<f32> = metrics.iter().map(|(_, value)| *value).collect();
        let max_value = values
            .iter()
            .cloned()
            .fold(0.0_f32, f32::max)
            .max(1.0);

        RadarChart::new(
            RadarAxes { labels, max_value },
            vec![RadarSeries {
                name: "Metrics".to_string(),
                color: Color::from_rgb(0.35, 0.62, 0.96),
                values,
            }],
        )
    }
}
