#[derive(Debug, Clone)]
pub enum Message {
    ToggleSidebar,
    Navigate(crate::screens::Page),
    Noop,
    StartImport,
    ImportPrepared(Result<Vec<crate::importing::NewHireRecord>, String>),
    ProcessedRecord(Result<crate::importing::ImportStep, String>),
    DecideDuplicate { number: i64, overwrite: bool },
    DecideAll { overwrite: bool },
    DecisionApplied(Result<crate::importing::ImportStep, String>),
    ReportSeriesLoaded(Result<Vec<(f32, f32)>, String>),
    ReportStateCountsLoaded(Result<Vec<(String, f32)>, String>),
    ReportEmployeeTypeLoaded(Result<Vec<(String, f32)>, String>),
    ReportHeatmapLoaded(Result<(Vec<String>, Vec<String>, Vec<Vec<f32>>), String>),
    ReportRadarLoaded(Result<Vec<(String, f32)>, String>),
    ReportBreachRateLoaded(Result<(f32, f32), String>),
}
