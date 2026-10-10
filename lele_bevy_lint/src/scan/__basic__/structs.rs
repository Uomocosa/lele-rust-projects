use crate::scan::basic::enums::PreviewKind;

pub struct FoundVisual {
    pub visual: String,
    pub line: usize,
    pub owner: String,
}

pub struct Preview {
    pub name: String,
    pub kind: PreviewKind,
    pub line: usize,
    pub is_ignored: bool,
    pub is_routed_through_harness: bool,
    pub idents: Vec<String>,
}
