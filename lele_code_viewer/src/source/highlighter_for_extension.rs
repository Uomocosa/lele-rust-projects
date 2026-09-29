use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::source;

pub fn highlighter_for_extension(ext: &str) -> source::Highlighter {
    let syntax_set = SyntaxSet::load_defaults_nonewlines();
    let syntax = syntax_set
        .find_syntax_by_extension(ext)
        .cloned()
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text().clone());
    let theme_set = ThemeSet::load_defaults();
    let theme = theme_set.themes.get("base16-ocean.dark").cloned();
    source::Highlighter {
        syntax_set,
        syntax,
        theme,
    }
}

#[cfg(test)]
mod tests {
    use super::highlighter_for_extension;

    #[test]
    fn test_usage() {
        let rust = highlighter_for_extension("rs");
        assert_eq!(rust.syntax.name, "Rust");
        let unknown = highlighter_for_extension("nope-unknown");
        assert!(unknown.theme.is_some());
        let toml = highlighter_for_extension("toml");
        assert_eq!(toml.theme.is_some(), rust.theme.is_some());
    }
}
