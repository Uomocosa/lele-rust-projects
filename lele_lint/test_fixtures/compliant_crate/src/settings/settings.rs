use super::settings_load;

pub struct Settings {
    pub root: String,
    pub verbose: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root: String::new(),
            verbose: false,
        }
    }
}

#[rustfmt::skip]
impl Settings {
    pub fn load() -> Self { settings_load::load() }
}

#[cfg(test)]
mod tests {
    use crate::settings;

    #[test]
    fn test_usage() {
        let s = settings::Settings::load();
        assert!(!s.verbose);
    }
}
