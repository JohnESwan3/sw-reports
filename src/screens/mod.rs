pub mod home;
pub mod import;
pub mod reports;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Page {
    Import,
    Home,
    Reports,
}

impl Page {
    pub fn label(&self) -> &'static str {
        match self {
            Page::Import => "Import",
            Page::Home => "Home",
            Page::Reports => "Reports",
        }
    }
}
