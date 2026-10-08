use std::collections::HashSet;

pub(crate) fn longest_type_prefix<'a>(stem: &'a str, known: &HashSet<String>) -> Option<&'a str> {
    stem.match_indices('_')
        .rev()
        .find_map(|(pos, _)| stem.get(..pos).filter(|prefix| known.contains(*prefix)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::longest_type_prefix;

    fn known(names: &[&str]) -> HashSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn test_usage() {
        let set = known(&["freenet_client", "freenet", "config"]);
        assert_eq!(
            longest_type_prefix("freenet_client_connect", &set),
            Some("freenet_client")
        );
        assert_eq!(longest_type_prefix("config_new", &set), Some("config"));
        assert_eq!(longest_type_prefix("bevy_systems", &set), None);
        assert_eq!(longest_type_prefix("plain", &set), None);
    }
}
