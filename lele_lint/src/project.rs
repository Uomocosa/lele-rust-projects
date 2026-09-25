use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use super::project_apply_layout;
use super::project_discover;
use super::project_find_cargo_root;
use super::project_get_parsed;
use crate::Config;
use crate::Dunder;
use crate::Entry;
use crate::Error;
use crate::Layout;
use crate::ModuleInfoMap;

#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    pub src_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub module_info: ModuleInfoMap,
    pub parsed_files: HashMap<PathBuf, syn::File>,
    pub layout: Layout,
    pub dunder: Dunder,
    pub container_placement: bool,
    pub methods_dir: Option<PathBuf>,
    pub methods_entries: Vec<Entry>,
    pub methods_parsed_files: HashMap<PathBuf, syn::File>,
}

#[rustfmt::skip]
impl Project {
    pub fn get_parsed(&self, rel_path: &Path) -> Option<&syn::File> {
        project_get_parsed::get_parsed(self, rel_path)
    }
    pub fn apply_layout(&mut self, config: &Config) -> Result<(), Error> {
        project_apply_layout::apply_layout(self, config)
    }
}

#[rustfmt::skip]
impl Project {
    pub fn discover(start_dir: Option<&Path>, scan_folders: Option<&[String]>) -> Result<Self, Error> {
        project_discover::discover(start_dir, scan_folders)
    }
    pub fn find_cargo_root(start: &Path) -> Result<PathBuf, Error> {
        project_find_cargo_root::find_cargo_root(start)
    }
}

// no test_usage necessary
