use poise::CreateReply;
use serenity::{
    all::ComponentInteraction,
    builder::{
        CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    http::Http,
    model::colour::Color,
};
use tracing::error;

use crate::data::{Context, Error};

/// An embed styled as a normal response
pub fn ok_embed(description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .description(description)
        .color(Color::DARK_GREEN)
}

/// An embed styled as an error response
pub fn error_embed(description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .description(description)
        .color(Color::DARK_RED)
}

/// Reply to a slash command with the given embed. Works whether or not the
/// command has been deferred.
pub async fn reply(ctx: Context<'_>, embed: CreateEmbed) -> Result<(), Error> {
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Respond to a button press that has not been deferred with the given embed.
pub async fn respond_to_component(
    interaction: &ComponentInteraction,
    http: &Http,
    embed: CreateEmbed,
) {
    let message = CreateInteractionResponseMessage::new().embed(embed);

    if let Err(err) = interaction
        .create_response(http, CreateInteractionResponse::Message(message))
        .await
    {
        error!("Failed to send button response: {}", err);
    }
}

/// Respond to a deferred button press with the given embed.
pub async fn respond_to_followup_component(
    interaction: &ComponentInteraction,
    http: &Http,
    embed: CreateEmbed,
) {
    let message = CreateInteractionResponseFollowup::new().embed(embed);

    if let Err(err) = interaction.create_followup(http, message).await {
        error!("Failed to send followup response: {}", err);
    }
}
