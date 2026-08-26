use std::path::PathBuf;

pub struct Instance {
    pub index: usize,
    pub pid: u32,
    pub title: String,
    pub log_path: PathBuf,
}
