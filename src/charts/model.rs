use iced::{Color, Point, Vector};

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    pub color: Color,
    pub points: Vec<(f32, f32)>,
}

impl LineSeries {
    pub fn new(name: impl Into<String>, color: Color, points: Vec<(f32, f32)>) -> Self {
        Self {
            name: name.into(),
            color,
            points,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineChartConfig {
    pub padding: f32,
    pub grid_lines: usize,
}

impl Default for LineChartConfig {
    fn default() -> Self {
        Self {
            padding: 40.0,
            grid_lines: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BarPoint {
    pub label: String,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct BarSeries {
    pub name: String,
    pub color: Color,
    pub values: Vec<BarPoint>,
}

#[derive(Debug, Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct HeatmapGrid {
    pub x_labels: Vec<String>,
    pub y_labels: Vec<String>,
    pub values: Vec<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct RadarAxes {
    pub labels: Vec<String>,
    pub max_value: f32,
}

#[derive(Debug, Clone)]
pub struct RadarSeries {
    pub name: String,
    pub color: Color,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct CircleMetric {
    pub label: String,
    pub value: f32,
    pub max: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum ChartData {
    Line(Vec<LineSeries>),
    Bar(BarSeries),
    Pie(Vec<PieSlice>),
    Heatmap(HeatmapGrid),
    Radar(RadarAxes, Vec<RadarSeries>),
    Circle(CircleMetric),
}

#[derive(Debug, Clone, Copy)]
pub struct InteractionConfig {
    pub enable_hover: bool,
    pub enable_zoom: bool,
    pub enable_pan: bool,
    pub enable_selection: bool,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            enable_hover: true,
            enable_zoom: true,
            enable_pan: true,
            enable_selection: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InteractionState {
    pub zoom: f32,
    pub pan: Vector,
    pub selection_start: Option<Point>,
    pub selection_end: Option<Point>,
    pub pan_start: Option<Point>,
    pub pan_origin: Vector,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vector::new(0.0, 0.0),
            selection_start: None,
            selection_end: None,
            pan_start: None,
            pan_origin: Vector::new(0.0, 0.0),
        }
    }
}
