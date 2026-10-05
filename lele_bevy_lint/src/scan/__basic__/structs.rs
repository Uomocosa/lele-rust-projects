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
    pub ignored: bool,
    pub calls_run: bool,
    pub idents: Vec<String>,
}
