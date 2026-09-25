use crate::settings;

pub fn load() -> settings::Settings {
    settings::Settings::default()
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
