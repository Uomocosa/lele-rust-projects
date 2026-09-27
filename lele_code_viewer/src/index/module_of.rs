use std::path::Component;
use std::path::Path;

pub fn module_of(file: &Path) -> String {
    let mut comps: Vec<String> = Vec::new();
    for comp in file.components() {
        if let Component::Normal(os) = comp {
            comps.push(os.to_string_lossy().to_string());
        }
    }
    if matches!(comps.first().map(String::as_str), Some("src" | "methods")) {
        comps.remove(0);
    }
    let last = comps.pop().unwrap_or_default();
    let stem = last.strip_suffix(".rs").unwrap_or(&last).to_string();
    if !matches!(stem.as_str(), "mod" | "lib" | "main") {
        comps.push(stem);
    }
    comps.join("::")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::module_of;

    #[test]
    fn test_usage() {
        assert_eq!(module_of(Path::new("src/index/mod.rs")), "index");
        assert_eq!(module_of(Path::new("src/lib.rs")), "");
        assert_eq!(
            module_of(Path::new("src/index/build_index.rs")),
            "index::build_index"
        );
        assert_eq!(
            module_of(Path::new("methods/clicker/add.rs")),
            "clicker::add"
        );
    }
}
