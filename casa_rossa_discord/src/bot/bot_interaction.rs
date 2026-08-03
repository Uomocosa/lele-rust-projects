use crate::bot;
use crate::scraper;
use serenity::all::{Context, Interaction};
use serenity::builder::{
    CreateAttachment, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use serenity::model::application::CommandInteraction;

pub async fn handle(bot: &bot::Bot, ctx: &Context, interaction: Interaction) {
    let command = match interaction {
        Interaction::Command(cmd) if cmd.data.name == "menu" => cmd,
        _ => return,
    };

    defer(&command, ctx).await;

    let images = match bot.scraper.fetch().await {
        Ok(imgs) if imgs.is_empty() => {
            respond_error(&command, ctx, "Nessuna immagine trovata nel menu").await;
            return;
        }
        Ok(imgs) => imgs,
        Err(e) => {
            respond_error(&command, ctx, &format!("Errore nel recupero del menu: {e}")).await;
            return;
        }
    };

    respond_menu(&command, ctx, &images).await;
}

async fn defer(command: &CommandInteraction, ctx: &Context) {
    let response = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new());
    if let Err(e) = command.create_response(&ctx.http, response).await {
        tracing::error!(target: "casa_rossa", error = %e, "failed to defer interaction");
    }
}

async fn respond_error(command: &CommandInteraction, ctx: &Context, msg: &str) {
    let response = EditInteractionResponse::new().content(msg);
    if let Err(e) = command.edit_response(&ctx.http, response).await {
        tracing::error!(target: "casa_rossa", error = %e, "failed to respond error");
    }
}

async fn respond_menu(command: &CommandInteraction, ctx: &Context, images: &[scraper::ImageEntry]) {
    let mut response =
        EditInteractionResponse::new().add_embed(CreateEmbed::new().title("Menu del giorno"));

    for img in images {
        response =
            response.new_attachment(CreateAttachment::bytes(img.data.clone(), &img.filename));
    }

    if let Err(e) = command.edit_response(&ctx.http, response).await {
        tracing::error!(target: "casa_rossa", error = %e, "failed to send menu");
    }
}
