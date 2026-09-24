use std::collections::BTreeMap;
use std::fmt::Write;

use super::DeclaredType;

pub(crate) fn root_index_content(types: &BTreeMap<String, DeclaredType>) -> String {
    let mut out = String::new();
    for (type_snake, declared) in types {
        if let Some(cfg) = combined_cfg(&declared.cfgs) {
            let _ = writeln!(out, "#[cfg({cfg})]");
        }
        let _ = writeln!(out, "pub mod {type_snake};");
    }
    out
}

// needed helper: combine a type's cfg predicates into one `cfg(...)` argument
fn combined_cfg(cfgs: &[String]) -> Option<String> {
    match cfgs {
        [] => None,
        [single] => Some(single.clone()),
        many => Some(format!("all({})", many.join(", "))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use super::root_index_content;
    use crate::common::DeclaredType;

    #[test]
    fn test_usage() {
        let mut types = BTreeMap::new();
        types.insert("click_counter".to_string(), DeclaredType::default());
        assert_eq!(root_index_content(&types), "pub mod click_counter;\n");
    }

    #[test]
    fn test_usage_cfg_is_propagated() {
        let mut types = BTreeMap::new();
        types.insert(
            "client".to_string(),
            DeclaredType {
                methods: BTreeSet::default(),
                cfgs: vec!["feature = \"room_lobby\"".to_string()],
            },
        );
        assert_eq!(
            root_index_content(&types),
            "#[cfg(feature = \"room_lobby\")]\npub mod client;\n"
        );
    }

    #[test]
    fn test_usage_multiple_cfgs_are_combined() {
        let mut types = BTreeMap::new();
        types.insert(
            "client".to_string(),
            DeclaredType {
                methods: BTreeSet::default(),
                cfgs: vec!["feature = \"a\"".to_string(), "unix".to_string()],
            },
        );
        assert_eq!(
            root_index_content(&types),
            "#[cfg(all(feature = \"a\", unix))]\npub mod client;\n"
        );
    }
}
