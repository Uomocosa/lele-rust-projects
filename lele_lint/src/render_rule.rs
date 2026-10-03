use std::fmt::Write;

use crate::Checker;
use crate::ExampleFile;

pub fn render_rule(checker: &dyn Checker) -> String {
    let doc = checker.doc();
    let mut out = String::new();
    let _ = writeln!(out, "### {} `{}`\n", checker.code(), checker.name());
    let _ = writeln!(out, "{}\n", doc.summary);
    let _ = writeln!(out, "**Why:** {}\n", doc.why);
    let _ = writeln!(out, "**Bad** (reports {}):\n", checker.code());
    out.push_str(&render_files(doc.bad));
    let _ = writeln!(out, "**Good:**\n");
    out.push_str(&render_files(doc.good));
    out
}

// needed helper: one fenced block per example file, labelled with its path
fn render_files(files: &[ExampleFile]) -> String {
    let mut out = String::new();
    for file in files {
        let lang = if file.path.ends_with(".toml") {
            "toml"
        } else {
            "rust"
        };
        let _ = writeln!(out, "`{}`\n", file.path);
        let _ = writeln!(out, "```{lang}\n{}```\n", file.source);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::render_rule;
    use crate::checkers::build_checkers;

    #[test]
    fn test_usage() {
        let checkers = build_checkers();
        let text = render_rule(checkers[0].as_ref());
        assert!(text.starts_with(&format!("### {}", checkers[0].code())));
        assert!(text.contains("**Bad**"));
        assert!(text.contains("```rust"));
    }
}
