use std::path::PathBuf;

pub struct Instance {
    pub index: usize,
    pub pid: u32,
    pub log_path: PathBuf,
    pub identity_dir: PathBuf,
}
