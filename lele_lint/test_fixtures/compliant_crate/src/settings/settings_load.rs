use super::settings::Settings;

pub fn load() -> Settings {
    Settings::default()
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn test_usage() {
        let s = load();
        assert!(s.root.is_empty());
    }
}
