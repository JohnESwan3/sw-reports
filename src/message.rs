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
}
