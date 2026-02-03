#[derive(Debug, Clone)]
pub enum Message {
    ToggleSidebar,
    Navigate(crate::screens::Page),
}
