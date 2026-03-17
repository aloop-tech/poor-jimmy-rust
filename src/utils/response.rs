use serenity::{
    all::{CommandInteraction, ComponentInteraction},
    builder::{
        CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    http::Http,
    model::colour::Color,
};
use tracing::error;

use crate::components::music_buttons::create_music_buttons;

/// Respond to a CommandInteraction with the given CreateEmbed.
///
/// This assumes the command has not been deferred or responded to yet. If the
/// command may have been deferred use `respond_to_follow` instead.
pub async fn respond_to_command(
    command: &CommandInteraction,
    http: &Http,
    content: String,
    include_buttons: bool,
) {
    let embed = CreateEmbed::new()
        .color(Color::DARK_GREEN)
        .description(content);

    let mut message = CreateInteractionResponseMessage::new().embed(embed);

    if include_buttons {
        message = message.components(create_music_buttons());
    }

    let response = CreateInteractionResponse::Message(message);

    if let Err(err) = command.create_response(http, response).await {
        error!("Failed to send command response: {}", err);
    }
}

pub async fn respond_to_error(command: &CommandInteraction, http: &Http, content: String) {
    let embed = CreateEmbed::new()
        .color(Color::DARK_RED)
        .description(content);

    let message = CreateInteractionResponseMessage::new().embed(embed);
    let response = CreateInteractionResponse::Message(message);

    if let Err(err) = command.create_response(http, response).await {
        error!("Failed to send error response: {}", err);
    }
}

pub async fn respond_to_button(
    command: &ComponentInteraction,
    http: &Http,
    content: String,
    include_buttons: bool,
) {
    let embed = CreateEmbed::new()
        .color(Color::DARK_GREEN)
        .description(content);

    let mut message = CreateInteractionResponseMessage::new().embed(embed);

    if include_buttons {
        message = message.components(create_music_buttons());
    }

    let response = CreateInteractionResponse::Message(message);

    if let Err(err) = command.create_response(http, response).await {
        error!("Failed to send button response: {}", err);
    }
}

pub async fn respond_to_error_button(command: &ComponentInteraction, http: &Http, content: String) {
    let embed = CreateEmbed::new()
        .color(Color::DARK_RED)
        .description(content);

    let message = CreateInteractionResponseMessage::new().embed(embed);
    let response = CreateInteractionResponse::Message(message);

    if let Err(err) = command.create_response(http, response).await {
        error!("Failed to send button error response: {}", err);
    }
}

/// Respond to a deferred CommandInteraction with the given
/// CreateEmbed.
///
/// This assumes the command has been deferred. If the command is not deferred
/// use `respond_to_command` instead.
pub async fn respond_to_followup(
    command: &CommandInteraction,
    http: &Http,
    content: CreateEmbed,
    include_buttons: bool,
) {
    let mut message = CreateInteractionResponseFollowup::new().embed(content);

    if include_buttons {
        message = message.components(create_music_buttons());
    }

    if let Err(err) = command.create_followup(http, message).await {
        error!("Failed to send followup response: {}", err);
    }
}

/// Respond to a deferred ComponentInteraction with the given
/// CreateEmbed.
///
/// This assumes the component interaction has been deferred.
pub async fn respond_to_followup_component(
    interaction: &ComponentInteraction,
    http: &Http,
    content: CreateEmbed,
    include_buttons: bool,
) {
    let mut message = CreateInteractionResponseFollowup::new().embed(content);

    if include_buttons {
        message = message.components(create_music_buttons());
    }

    if let Err(err) = interaction.create_followup(http, message).await {
        error!("Failed to send followup response: {}", err);
    }
}
