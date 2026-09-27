use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::html;

pub fn render_md(text: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    let parser = Parser::new_ext(text, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::render_md;

    #[test]
    fn test_usage() {
        let html = render_md("# Title\n\n- a\n- b\n");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<li>a</li>"));
    }
}
