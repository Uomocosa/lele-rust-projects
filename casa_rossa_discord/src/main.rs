use casa_rossa_discord::bot;
use casa_rossa_discord::config;

#[tokio::main]
async fn main() -> casa_rossa_discord::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _ = dotenvy::dotenv();

    let cfg = config::Config::from_env()?;
    let bot = bot::Bot::new(cfg);
    bot.run().await
}
