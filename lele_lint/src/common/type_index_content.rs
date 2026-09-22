use std::collections::BTreeSet;
use std::fmt::Write;

pub(crate) fn type_index_content(methods: &BTreeSet<String>) -> String {
    let mut out = String::new();
    for method in methods {
        let _ = writeln!(out, "mod {method};");
    }
    for method in methods {
        let _ = writeln!(out, "pub use {method}::{method};");
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::type_index_content;

    #[test]
    fn test_usage() {
        let methods: BTreeSet<String> = ["add".to_string(), "increment".to_string()].into();
        assert_eq!(
            type_index_content(&methods),
            "mod add;\nmod increment;\npub use add::add;\npub use increment::increment;\n"
        );
    }
}
