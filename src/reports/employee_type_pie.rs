use std::path::PathBuf;

use iced::Color;

use crate::charts::{PieChart, PieSlice};
use crate::data::employee_type_counts;

pub struct EmployeeTypePieReport;

impl EmployeeTypePieReport {
    pub fn title() -> &'static str {
        "Employee Type Distribution"
    }

    pub fn subtitle() -> &'static str {
        "Share of records by employee type"
    }

    pub async fn load(db_path: PathBuf) -> Result<Vec<(String, f32)>, String> {
        employee_type_counts::load_series(db_path).await
    }

    pub fn chart(points: &[(String, f32)]) -> PieChart {
        let palette = [
            Color::from_rgb(0.35, 0.62, 0.96),
            Color::from_rgb(0.42, 0.85, 0.53),
            Color::from_rgb(0.95, 0.67, 0.29),
            Color::from_rgb(0.89, 0.40, 0.40),
            Color::from_rgb(0.73, 0.54, 0.96),
            Color::from_rgb(0.35, 0.85, 0.83),
        ];

        let slices = points
            .iter()
            .enumerate()
            .map(|(index, (label, value))| PieSlice {
                label: label.clone(),
                value: *value,
                color: palette[index % palette.len()],
            })
            .collect();

        PieChart::new(slices)
    }
}
