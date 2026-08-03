use super::bot_interaction;
use super::bot_ready;
use super::bot_run;
use crate::config;
use crate::scraper;
use serenity::all::{Context, EventHandler, Interaction, Ready};
use serenity::async_trait;

pub struct Bot {
    pub scraper: scraper::MenuScraper,
    pub config: config::Config,
}

#[allow(clippy::derivable_impls)]
impl Default for Bot {
    fn default() -> Self {
        Self {
            scraper: scraper::MenuScraper::default(),
            config: config::Config::default(),
        }
    }
}

#[rustfmt::skip]
impl Bot {
    pub fn new(config: config::Config) -> Self { Self { scraper: scraper::MenuScraper::new(&config), config } }
    pub async fn run(self) -> crate::Result<()> { bot_run::run(self).await }
}

#[async_trait]
#[rustfmt::skip]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) { bot_ready::handle(self, &ctx, ready).await }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) { bot_interaction::handle(self, &ctx, interaction).await }
}
