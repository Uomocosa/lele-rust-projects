pub struct Config(pub Option<String>);

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_usage() {
        let c = Config(Some(String::new()));
        let _ = c;
    }
}
