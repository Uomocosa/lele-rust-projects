use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::source;

pub fn highlighter_new() -> source::Highlighter {
    let syntax_set = SyntaxSet::load_defaults_nonewlines();
    let syntax = syntax_set
        .find_syntax_by_extension("rs")
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
    use super::highlighter_new;

    #[test]
    fn test_usage() {
        let hl = highlighter_new();
        assert!(hl.theme.is_some());
    }
}
