pub(crate) fn to_snake_case(pascal: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pascal.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let preceded_by_lower = i > 0
                && chars
                    .get(i.saturating_sub(1))
                    .is_some_and(|p| p.is_lowercase());
            let followed_by_lower = chars
                .get(i.saturating_add(1))
                .is_some_and(|p| p.is_lowercase())
                && i.saturating_add(1) < len;
            let preceded_by_upper = i > 0
                && chars
                    .get(i.saturating_sub(1))
                    .is_some_and(|p| p.is_uppercase());

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

    #[test]
    fn test_usage_acronym_boundaries() {
        assert_eq!(to_snake_case("P2PPlugin"), "p2p_plugin");
        assert_eq!(to_snake_case("P2PEvents"), "p2p_events");
        assert_eq!(to_snake_case("NetworkId"), "network_id");
        assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
        assert_eq!(to_snake_case("IOStream"), "io_stream");
    }
}
