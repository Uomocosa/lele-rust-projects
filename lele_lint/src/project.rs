use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;

use crate::Config;
use crate::Dunder;
use crate::Entry;
use crate::Error;
use crate::ModuleInfoMap;

#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    pub src_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub module_info: ModuleInfoMap,
    pub parsed_files: HashMap<PathBuf, syn::File>,
    pub dunder: Dunder,
    pub methods_dir: Option<PathBuf>,
    pub methods_entries: Vec<Entry>,
    pub methods_parsed_files: HashMap<PathBuf, syn::File>,
}

#[atomic_delegates]
impl Project {
    pub fn apply_layout(&mut self, config: &Config) -> Result<(), Error> {}
    pub fn discover(
        start_dir: Option<&Path>,
        scan_folders: Option<&[String]>,
    ) -> Result<Self, Error> {
    }
    pub fn find_cargo_root(start: &Path) -> Result<PathBuf, Error> {}
}

#[rustfmt::skip]
impl Project {
    pub fn get_parsed(&self, rel_path: &Path) -> Option<&syn::File> {
        self.parsed_files.get(rel_path)
    }
}

// no test_usage necessary
