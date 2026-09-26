use crate::settings;

pub fn load() -> settings::Settings {
    let root = String::new();
    settings::Settings { root, verbose: false }
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
