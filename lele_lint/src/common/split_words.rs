pub(crate) fn split_words(ident: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous: Option<char> = None;
    let mut chars = ident.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '_' || character == '-' {
            flush(&mut current, &mut words);
            previous = None;
            continue;
        }
        if is_boundary(previous, character, chars.peek().copied()) {
            flush(&mut current, &mut words);
        }
        current.push(character);
        previous = Some(character);
    }
    flush(&mut current, &mut words);
    words
}

// needed helper: a word starts at a camel edit or an alpha/digit edit
fn is_boundary(previous: Option<char>, current: char, next: Option<char>) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    if previous.is_alphanumeric()
        && current.is_alphanumeric()
        && previous.is_ascii_digit() != current.is_ascii_digit()
    {
        return true;
    }
    if current.is_ascii_uppercase() {
        if previous.is_ascii_lowercase() || previous.is_ascii_digit() {
            return true;
        }
        if previous.is_ascii_uppercase() && next.is_some_and(|n| n.is_ascii_lowercase()) {
            return true;
        }
    }
    false
}

// needed helper: commit the accumulated word, lowercased
fn flush(current: &mut String, words: &mut Vec<String>) {
    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
        current.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::split_words;

    #[test]
    fn test_usage() {
        assert_eq!(split_words("run_board"), vec!["run", "board"]);
        assert_eq!(split_words("BoardState"), vec!["board", "state"]);
        assert_eq!(split_words("HTTPServer"), vec!["http", "server"]);
        assert_eq!(split_words("keyboard"), vec!["keyboard"]);
        assert_eq!(split_words("utf8_reader"), vec!["utf", "8", "reader"]);
        assert_eq!(split_words("kebab-case"), vec!["kebab", "case"]);
        assert_eq!(split_words(""), Vec::<String>::new());
    }
}
