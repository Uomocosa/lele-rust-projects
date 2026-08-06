use std::path::Path;

use super::project::Project;

pub(crate) fn get_parsed<'a>(project: &'a Project, rel_path: &Path) -> Option<&'a syn::File> {
    project.parsed_files.get(rel_path)
}

// no test_usage necessary
