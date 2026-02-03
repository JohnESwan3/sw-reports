use std::path::PathBuf;

use crate::charts::{HeatmapChart, HeatmapGrid, LineChartConfig};
use crate::data::heatmap_site_employee;

pub struct SiteEmployeeHeatmapReport;

impl SiteEmployeeHeatmapReport {
    pub fn title() -> &'static str {
        "Site vs Employee Type"
    }

    pub fn subtitle() -> &'static str {
        "Counts per site and employee type"
    }

    pub async fn load(
        db_path: PathBuf,
    ) -> Result<(Vec<String>, Vec<String>, Vec<Vec<f32>>), String> {
        heatmap_site_employee::load_grid(db_path).await
    }

    pub fn chart(grid: HeatmapGrid) -> HeatmapChart {
        HeatmapChart::new(grid).with_config(LineChartConfig {
            padding: 50.0,
            grid_lines: 4,
        })
    }
}
