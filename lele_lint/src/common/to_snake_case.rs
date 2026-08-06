pub(crate) fn to_snake_case(pascal: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pascal.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let preceded_by_lower = i > 0 && chars[i - 1].is_lowercase();
            let followed_by_lower = i + 1 < len && chars[i + 1].is_lowercase();
            let preceded_by_upper = i > 0 && chars[i - 1].is_uppercase();

            if preceded_by_lower || (followed_by_lower && preceded_by_upper) {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::to_snake_case;

    #[test]
    fn test_usage() {
        assert_eq!(to_snake_case("ConstructorNoSkip"), "constructor_no_skip");
        assert_eq!(to_snake_case("DomainImport"), "domain_import");
        assert_eq!(to_snake_case("SnakeCaseFiles"), "snake_case_files");
        assert_eq!(to_snake_case("NoPositional"), "no_positional");
        assert_eq!(to_snake_case("Player"), "player");
        assert_eq!(to_snake_case("PlayerEvent"), "player_event");
    }
}
