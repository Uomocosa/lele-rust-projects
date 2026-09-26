use crate::project::Project;
use crate::project_parse_source_files;
use crate::project_walk_entries;
use crate::Config;
use crate::Error;
use crate::Layout;

pub(crate) fn apply_layout(project: &mut Project, config: &Config) -> Result<(), Error> {
    let layout = config.layout();
    project.layout = layout;
    project.dunder = config.dunder();
    project.container_placement = config.checker_enabled("container_placement");

    if layout != Layout::Methods {
        return Ok(());
    }

    let methods_dir = project.root.join("methods");
    if !methods_dir.is_dir() {
        return Ok(());
    }

    let entries = project_walk_entries::walk_entries(&methods_dir, &methods_dir)?;
    let parsed_files = project_parse_source_files::parse_source_files(&methods_dir, &entries);
    project.methods_dir = Some(methods_dir);
    project.methods_entries = entries;
    project.methods_parsed_files = parsed_files;
    Ok(())
}

// no test_usage necessary
