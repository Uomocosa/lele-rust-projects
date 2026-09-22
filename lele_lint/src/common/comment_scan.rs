use rustc_lexer::tokenize;
use rustc_lexer::TokenKind;

pub(crate) struct CommentHit {
    pub line: usize,
    pub text: String,
    pub block: bool,
}

pub(crate) fn find_comments(source: &str) -> Vec<CommentHit> {
    let mut hits = Vec::new();
    let mut offset: usize = 0;
    for token in tokenize(source) {
        let len = token.len;
        let start = offset;
        offset = offset.saturating_add(len);
        match token.kind {
            TokenKind::LineComment => {
                let text = source
                    .get(start..offset)
                    .unwrap_or("")
                    .trim_end()
                    .to_string();
                hits.push(CommentHit {
                    line: line_of(source, start),
                    text,
                    block: false,
                });
            }
            TokenKind::BlockComment { .. } => {
                hits.push(CommentHit {
                    line: line_of(source, start),
                    text: String::new(),
                    block: true,
                });
            }
            _ => {}
        }
    }
    hits
}

// needed helper: 1-based line number for a byte offset
fn line_of(source: &str, offset: usize) -> usize {
    source
        .get(..offset)
        .map_or(1, |prefix| prefix.matches('\n').count().saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::find_comments;

    #[test]
    fn test_usage() {
        let source = "let x = \"http://not-a-comment\"; // real\n/* block */\nfn f() {}\n";
        let hits = find_comments(source);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line, 1);
        assert!(!hits[0].block);
        assert!(hits[0].text.contains("real"));
        assert!(hits[1].block);
        assert_eq!(hits[1].line, 2);
    }

    #[test]
    fn test_usage_doc_comments() {
        let hits = find_comments("/// doc\nfn f() {}\n");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line, 1);
    }
}
