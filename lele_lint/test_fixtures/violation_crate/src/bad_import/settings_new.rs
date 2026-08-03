use super::settings::Settings;

pub fn new() -> Settings {
    Settings { high_score: crate::bad_getter::Score { value: 0 } }
}
