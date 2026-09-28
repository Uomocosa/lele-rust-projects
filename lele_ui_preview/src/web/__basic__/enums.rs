#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Navigate {
        location: String,
        label: String,
    },
    Click {
        index: usize,
        label: String,
    },
    Fill {
        index: usize,
        text: String,
        label: String,
    },
    Key {
        key: String,
    },
    ScrollBottom,
}
