use std::path::Path;

use crate::common;
use crate::Error;
use crate::Project;

pub fn sync_methods(project: &Project) -> Result<(), Error> {
    let Some(methods_dir) = &project.methods_dir else {
        return Err(Error::NoMethodsDir);
    };
    let declared = common::collect_declared(project);
    write_if_changed(
        &methods_dir.join("mod.rs"),
        &common::root_index_content(&declared),
    )?;
    for (type_snake, declared_type) in &declared {
        let dir = methods_dir.join(type_snake);
        std::fs::create_dir_all(&dir)?;
        write_if_changed(
            &dir.join("mod.rs"),
            &common::type_index_content(&declared_type.methods),
        )?;
    }
    Ok(())
}

// needed helper: write only when content differs, keeping the command idempotent
fn write_if_changed(path: &Path, content: &str) -> Result<(), Error> {
    if std::fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_if_changed;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mod.rs");
        write_if_changed(&path, "pub mod a;\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "pub mod a;\n");
        write_if_changed(&path, "pub mod a;\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "pub mod a;\n");
    }
}
