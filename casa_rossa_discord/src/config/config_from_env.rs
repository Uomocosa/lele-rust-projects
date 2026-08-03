use crate::config;

pub fn from_env() -> crate::Result<config::Config> {
    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| crate::Error::Config("DISCORD_TOKEN not set".into()))?;
    let guild_id: u64 = std::env::var("GUILD_ID")
        .map_err(|_| crate::Error::Config("GUILD_ID not set".into()))?
        .parse()
        .map_err(|e| crate::Error::Config(format!("GUILD_ID parse error: {e}")))?;
    let base_url = std::env::var("MENU_BASE_URL")
        .unwrap_or_else(|_| "http://www.casarossailfuturista.it".into());
    Ok(config::Config {
        discord_token: token,
        guild_id,
        menu_base_url: base_url,
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_usage() {
        let _ = super::from_env();
    }
}
