use atomic_delegate_macros::atomic_delegates;

use crate::settings;

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

#[atomic_delegates]
impl Settings {
    pub fn load() -> Self {}
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
