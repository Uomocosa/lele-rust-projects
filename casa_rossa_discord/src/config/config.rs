use super::config_from_env;

pub struct Config {
    pub discord_token: String,
    pub guild_id: u64,
    pub menu_base_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discord_token: String::new(),
            guild_id: 0,
            menu_base_url: "http://www.casarossailfuturista.it".into(),
        }
    }
}

#[rustfmt::skip]
impl Config {
    pub fn from_env() -> crate::Result<Self> { config_from_env::from_env() }
}
