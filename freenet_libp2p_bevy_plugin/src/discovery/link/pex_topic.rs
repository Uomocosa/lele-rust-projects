use super::super::constants;

#[must_use]
pub fn pex_topic(namespace: &str) -> String {
    format!("{namespace}/{}", constants::PEX_TOPIC_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::pex_topic;

    #[test]
    fn test_usage() {
        assert_eq!(pex_topic("ns"), "ns/pex");
    }
}
