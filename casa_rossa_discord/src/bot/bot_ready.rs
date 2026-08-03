use crate::bot;
use serenity::all::{Context, CreateCommand, GuildId, Ready};

pub async fn handle(bot: &bot::Bot, ctx: &Context, _ready: Ready) {
    let cmd = CreateCommand::new("menu").description("Mostra il menu del giorno");
    let guild = GuildId::new(bot.config.guild_id);
    if let Err(e) = guild.set_commands(&ctx.http, vec![cmd]).await {
        tracing::error!(target: "casa_rossa", error = %e, "failed to register slash command");
        return;
    }
    tracing::info!(target: "casa_rossa", "bot ready — /menu registered");
}
