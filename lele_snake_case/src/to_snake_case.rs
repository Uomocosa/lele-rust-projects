use super::constants;

#[must_use]
pub fn to_snake_case(pascal: &str) -> String {
    let chars: Vec<char> = pascal.chars().collect();
    let mut result = String::new();
    let mut skip_until = 0usize;

    for (i, &c) in chars.iter().enumerate() {
        if i < skip_until {
            continue;
        }
        if let Some(acronym) = acronym_at(&chars, i) {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            result.push_str(&acronym.to_ascii_lowercase());
            skip_until = i.saturating_add(acronym.chars().count());
            continue;
        }
        if c.is_uppercase() {
            let preceded_by_lower = i > 0
                && chars
                    .get(i.saturating_sub(1))
                    .is_some_and(|p| p.is_lowercase());
            let preceded_by_upper = i > 0
                && chars
                    .get(i.saturating_sub(1))
                    .is_some_and(|p| p.is_uppercase());
            let followed_by_lower = chars
                .get(i.saturating_add(1))
                .is_some_and(|p| p.is_lowercase());
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

// needed helper: longest known acronym starting at `i` on a word boundary
fn acronym_at(chars: &[char], i: usize) -> Option<&'static str> {
    let at_boundary = i == 0
        || chars
            .get(i.saturating_sub(1))
            .is_some_and(|p| !p.is_uppercase());
    if !at_boundary {
        return None;
    }
    constants::ACRONYMS
        .iter()
        .copied()
        .find(|&acronym| matches_acronym(chars, i, acronym))
}

// needed helper: does `chars` from `start` begin with `acronym` and end on a word boundary
fn matches_acronym(chars: &[char], start: usize, acronym: &str) -> bool {
    let matched = acronym
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(start.saturating_add(offset)) == Some(&expected));
    if !matched {
        return false;
    }
    let end = start.saturating_add(acronym.chars().count());
    chars.get(end).is_none_or(|c| !c.is_lowercase())
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
        assert_eq!(
            to_snake_case("P2PRoomDiscoveryPlugin"),
            "p2p_room_discovery_plugin"
        );
        assert_eq!(to_snake_case("P2PEvents"), "p2p_events");
        assert_eq!(to_snake_case("P2P"), "p2p");
        assert_eq!(to_snake_case("MyP2PThing"), "my_p2p_thing");
        assert_eq!(to_snake_case("NetworkId"), "network_id");
        assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
        assert_eq!(to_snake_case("IOStream"), "io_stream");
    }

    #[test]
    fn test_usage_lowercase_acronym_falls_back() {
        assert_eq!(to_snake_case("P2pPlugin"), "p2p_plugin");
        assert_eq!(to_snake_case("P2Plus"), "p2plus");
    }
}
