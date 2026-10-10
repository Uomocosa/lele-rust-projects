use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;

use crate::AllowWhitelistEntry;
use crate::BoundaryEntry;
use crate::Config;
use crate::DunderPath;
use crate::Entry;
use crate::Error;
use crate::ModuleInfoMap;
use crate::Origin;
use crate::ParseFailure;
use crate::ParsedSource;

#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    pub src_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub module_info: ModuleInfoMap,
    pub parsed_files: HashMap<PathBuf, syn::File>,
    pub dunder_paths: Vec<DunderPath>,
    pub clippy_allow_whitelist: Vec<AllowWhitelistEntry>,
    pub methods_dir: Option<PathBuf>,
    pub methods_entries: Vec<Entry>,
    pub methods_parsed_files: HashMap<PathBuf, syn::File>,
    pub example_entries: Vec<Entry>,
    pub example_parsed_files: HashMap<PathBuf, syn::File>,
    pub boundaries: Vec<BoundaryEntry>,
    pub parse_failures: Vec<ParseFailure>,
}

#[atomic_delegates]
impl Project {
    pub fn apply_layout(&mut self, config: &Config) -> Result<(), Error> {}
    pub fn discover(
        start_dir: Option<&Path>,
        scan_folders: Option<&[PathBuf]>,
    ) -> Result<Self, Error> {
    }
    pub fn find_cargo_root(start: &Path) -> Result<PathBuf, Error> {}
    pub fn sources(&self) -> impl Iterator<Item = ParsedSource<'_>> {}
}

#[rustfmt::skip]
impl Project {
    pub fn get_parsed(&self, rel_path: &Path) -> Option<&syn::File> {
        self.parsed_files.get(rel_path)
    }
    pub fn absolute_path(&self, origin: Origin, rel_path: &Path) -> PathBuf {
        match origin {
            Origin::Src => self.src_dir.join(rel_path),
            Origin::Methods => self.methods_dir.clone().unwrap_or_else(|| self.root.join("methods")).join(rel_path),
            Origin::Examples => self.root.join("examples").join(rel_path),
        }
    }
    pub fn content_sources(&self) -> impl Iterator<Item = ParsedSource<'_>> {
        self.sources()
            .filter(|source| !matches!(source.origin, Origin::Examples))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::Origin;
    use crate::Project;

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
        let origins: Vec<Origin> = project
            .content_sources()
            .map(|source| source.origin)
            .collect();
        assert_eq!(origins.len(), 2);
        assert!(!origins.contains(&Origin::Examples));
    }
}
