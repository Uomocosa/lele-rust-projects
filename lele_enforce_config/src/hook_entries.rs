pub fn hook_entries(nix_text: &str) -> Vec<String> {
    let mut in_hooks = false;
    let mut out = Vec::new();
    for line in nix_text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("git-hooks") {
            in_hooks = true;
        }
        if !in_hooks {
            continue;
        }
        if let Some(value) = trimmed
            .strip_prefix("entry = \"")
            .and_then(|rest| rest.strip_suffix("\";"))
        {
            out.push(unwrap_bash(value));
        }
    }
    out
}

fn unwrap_bash(value: &str) -> String {
    value
        .strip_prefix("bash -c '")
        .and_then(|inner| inner.strip_suffix('\''))
        .map_or_else(|| value.to_owned(), std::borrow::ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::hook_entries;

    #[test]
    fn test_usage() {
        let nix = "tasks = {};\ngit-hooks.hooks = {\n  a = {\n    entry = \"bash -c 'cd foo && devenv tasks run lele:clippy 2>&1'\";\n  };\n  b = {\n    entry = \"devenv tasks run lele:enforce-config 2>&1\";\n  };\n};\n";
        assert_eq!(
            hook_entries(nix),
            [
                "cd foo && devenv tasks run lele:clippy 2>&1".to_owned(),
                "devenv tasks run lele:enforce-config 2>&1".to_owned()
            ]
        );
        assert_eq!(hook_entries("tasks = {};\n"), Vec::<String>::new());
        assert_eq!(hook_entries(""), Vec::<String>::new());
    }
}
