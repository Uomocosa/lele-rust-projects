#[derive(Debug, Clone)]
pub struct Reexport {
    pub segments: Vec<String>,
    pub is_glob: bool,
}
