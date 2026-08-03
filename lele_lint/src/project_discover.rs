// no test_usage necessary
use super::project::Project;
use crate::error::Error;
use std::path::Path;

pub fn discover(start_dir: Option<&Path>) -> Result<Project, Error> {
    let cwd = std::env::current_dir()?;
    let search_from = start_dir.unwrap_or(&cwd);
    let root = crate::project_find_cargo_root::find_cargo_root(search_from)?;
    let src_dir = root.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return Err(Error::NoSrcDirectory(src_dir.display().to_string()));
    }
    let entries = super::project::walk_entries(&src_dir)?;
    let module_info = crate::module_info::ModuleInfo::build(&src_dir, &entries);
    let parsed_files = super::project::parse_source_files(&src_dir, &entries);
    Ok(Project {
        root,
        src_dir,
        entries,
        module_info,
        parsed_files,
    })
}
