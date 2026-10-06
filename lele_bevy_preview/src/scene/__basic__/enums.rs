use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Kind {
    #[default]
    System,
    Component,
    Plugin,
    App,
}

impl Kind {
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::System => "[system]",
            Self::Component => "[component]",
            Self::Plugin => "[plugin]",
            Self::App => "[app]",
        }
    }
}
