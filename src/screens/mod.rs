pub mod home;
pub mod reports;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Page {
    Home,
    Reports,
}

impl Page {
    pub fn label(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Reports => "Reports",
        }
    }
}
