use std::collections::BTreeMap;

use crate::common;
use crate::Project;

pub(crate) fn collect_declared(project: &Project) -> BTreeMap<String, common::DeclaredType> {
    let mut map: BTreeMap<String, common::DeclaredType> = BTreeMap::new();
    let cfg_map = common::module_cfgs::build(&project.module_info);
    for (rel_path, file) in &project.parsed_files {
        let Some(stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(primary) = common::primary_type_name(file, stem) else {
            continue;
        };
        let type_snake = common::to_snake_case(&primary);
        for item in &file.items {
            let syn::Item::Impl(impl_block) = item else {
                continue;
            };
            if impl_block.trait_.is_some() {
                continue;
            }
            if common::self_type_last(&impl_block.self_ty).as_deref() != Some(primary.as_str()) {
                continue;
            }
            if !common::has_atomic_delegates(&impl_block.attrs) {
                continue;
            }
            let entry = map.entry(type_snake.clone()).or_default();
            if entry.cfgs.is_empty() {
                entry.cfgs = common::file_cfgs(&cfg_map, rel_path);
            }
            for impl_item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    entry.methods.insert(method.sig.ident.to_string());
                }
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::collect_declared;
    use crate::Project;

    #[test]
    fn test_usage() {
        let mut project = Project {
            ..Project::default()
        };
        let file: syn::File = syn::parse_str(
            "pub struct ClickCounter(pub i32);\n#[atomic_delegates]\nimpl ClickCounter { pub fn add(&mut self) {} }",
        )
        .unwrap();
        project
            .parsed_files
            .insert(PathBuf::from("clicker/click_counter.rs"), file);
        let declared = collect_declared(&project);
        assert!(declared
            .get("click_counter")
            .is_some_and(|d| d.methods.contains("add")));
    }
}
