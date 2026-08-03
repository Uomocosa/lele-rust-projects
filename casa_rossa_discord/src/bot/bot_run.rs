use crate::bot;
use serenity::all::{Client, GatewayIntents};

pub async fn run(bot: bot::Bot) -> crate::Result<()> {
    let mut client = Client::builder(&bot.config.discord_token, GatewayIntents::empty())
        .event_handler(bot)
        .await?;

    client.start().await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_usage() {
        let cfg = crate::config::Config {
            discord_token: "invalid_token".into(),
            ..crate::config::Config::default()
        };
        let bot = crate::bot::Bot::new(cfg);
        let result = super::run(bot).await;
        assert!(result.is_err());
    }
}
