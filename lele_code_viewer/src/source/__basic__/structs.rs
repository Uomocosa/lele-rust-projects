pub struct Highlighter {
    pub syntax_set: syntect::parsing::SyntaxSet,
    pub syntax: syntect::parsing::SyntaxReference,
    pub theme: Option<syntect::highlighting::Theme>,
}

// no test_usage necessary
