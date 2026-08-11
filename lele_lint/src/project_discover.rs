use super::project::Project;
use crate::error;
use crate::module_info;
use crate::project_find_cargo_root;
use crate::project_parse_source_files;
use crate::project_walk_entries;
use std::path::Path;

pub fn discover(start_dir: Option<&Path>) -> Result<Project, error::Error> {
    let cwd = std::env::current_dir()?;
    let search_from = start_dir.unwrap_or(&cwd);
    let root = project_find_cargo_root::find_cargo_root(search_from)?;
    let src_dir = root.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return Err(error::Error::NoSrcDirectory(src_dir.display().to_string()));
    }
    let entries = project_walk_entries::walk_entries(&src_dir)?;
    let module_info = module_info::ModuleInfo::build(&src_dir, &entries);
    let parsed_files = project_parse_source_files::parse_source_files(&src_dir, &entries);
    Ok(Project {
        root,
        src_dir,
        entries,
        module_info,
        parsed_files,
    })
}

// no test_usage necessary
