pub(crate) fn to_pascal_case(snake: &str) -> String {
    snake
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::to_pascal_case;

    #[test]
    fn test_usage() {
        assert_eq!(to_pascal_case("client"), "Client");
        assert_eq!(to_pascal_case("events"), "Events");
        assert_eq!(to_pascal_case("history_chunk"), "HistoryChunk");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_usage_acronym_suffix_is_title_cased() {
        assert_eq!(to_pascal_case("https_proxy"), "HttpsProxy");
    }
}
