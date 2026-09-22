use std::collections::BTreeSet;
use std::fmt::Write;

pub(crate) fn root_index_content(types: &BTreeSet<String>) -> String {
    let mut out = String::new();
    for type_snake in types {
        let _ = writeln!(out, "pub mod {type_snake};");
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::root_index_content;

    #[test]
    fn test_usage() {
        let types: BTreeSet<String> = ["click_counter".to_string()].into();
        assert_eq!(root_index_content(&types), "pub mod click_counter;\n");
    }
}
