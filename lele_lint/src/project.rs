use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::project_discover;
use super::project_find_cargo_root;
use super::project_get_parsed;
use crate::entry;
use crate::error;
use crate::module_info;

pub struct Project {
    pub root: PathBuf,
    pub src_dir: PathBuf,
    pub entries: Vec<entry::Entry>,
    pub module_info: module_info::ModuleInfoMap,
    pub parsed_files: HashMap<PathBuf, syn::File>,
}

#[rustfmt::skip]
impl Project {
    pub fn get_parsed(&self, rel_path: &Path) -> Option<&syn::File> {
        project_get_parsed::get_parsed(self, rel_path)
    }
}

#[rustfmt::skip]
impl Project {
    pub fn discover(start_dir: Option<&Path>, scan_folders: Option<&[String]>) -> Result<Self, error::Error> {
        project_discover::discover(start_dir, scan_folders)
    }
    pub fn find_cargo_root(start: &Path) -> Result<PathBuf, error::Error> {
        project_find_cargo_root::find_cargo_root(start)
    }
}

// no test_usage necessary
