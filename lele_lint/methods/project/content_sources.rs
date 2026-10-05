use crate::Origin;
use crate::ParsedSource;
use crate::Project;

pub fn content_sources(project: &Project) -> impl Iterator<Item = ParsedSource<'_>> {
    let src = project
        .parsed_files
        .iter()
        .map(|(relative_path, file)| ParsedSource {
            origin: Origin::Src,
            relative_path,
            file,
        });
    let methods = project
        .methods_parsed_files
        .iter()
        .map(|(relative_path, file)| ParsedSource {
            origin: Origin::Methods,
            relative_path,
            file,
        });
    src.chain(methods)
}

#[cfg(test)]
mod tests {
    use super::content_sources;
    use crate::Origin;
    use crate::Project;
    use std::path::PathBuf;

    #[test]
    fn test_usage() {
        let mut project = Project::default();
        project
            .parsed_files
            .insert(PathBuf::from("a.rs"), syn::parse_str("").unwrap());
        project
            .methods_parsed_files
            .insert(PathBuf::from("b.rs"), syn::parse_str("").unwrap());
        project
            .example_parsed_files
            .insert(PathBuf::from("c.rs"), syn::parse_str("").unwrap());
        let origins: Vec<Origin> = content_sources(&project)
            .map(|source| source.origin)
            .collect();
        assert_eq!(origins.len(), 2);
        assert!(!origins.contains(&Origin::Examples));
    }
}
