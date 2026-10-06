use poise::{CreateReply, FrameworkError};
use serenity::all::{Context as SerenityContext, FullEvent, Interaction};
use tracing::{debug, error};

use crate::{
    commands,
    data::{Context, Data, Error},
    utils::response::{error_embed, respond_to_component},
};

/// Logs each slash command before it runs
pub async fn pre_command(ctx: Context<'_>) {
    let guild_id = ctx
        .guild_id()
        .map(|g| g.to_string())
        .unwrap_or_else(|| "DM".to_string());

    debug!(
        "Received command '{}' from user {} in guild {}",
        ctx.command().qualified_name,
        ctx.author().name,
        guild_id
    );
}

/// Handles gateway events poise doesn't route itself: the music control and
/// search result buttons.
pub async fn handle_event(
    ctx: &SerenityContext,
    event: &FullEvent,
    data: &Data,
) -> Result<(), Error> {
    let FullEvent::InteractionCreate {
        interaction: Interaction::Component(interaction),
    } = event
    else {
        return Ok(());
    };

    let button_id = interaction.data.custom_id.as_str();
    let user = &interaction.user;

    let Some(guild_id) = interaction.guild_id else {
        debug!(
            "Rejected button '{}' from user {} in DM",
            button_id, user.name
        );
        respond_to_component(
            interaction,
            &ctx.http,
            error_embed("Poor Jimmy only works in servers!"),
        )
        .await;
        return Ok(());
    };

    debug!(
        "Received button interaction '{}' from user {}",
        button_id, user.name
    );

    if button_id.starts_with("search_play_") {
        commands::search::handle_component(ctx, interaction, data, guild_id).await;
        return Ok(());
    }

    let embed = match button_id {
        "clear" => commands::clear::action(ctx, guild_id).await,
        "loop" => commands::r#loop::action(ctx, guild_id).await,
        "pause" => commands::pause::action(ctx, guild_id).await,
        "resume" => commands::resume::action(ctx, guild_id).await,
        "skip" => commands::skip::action(ctx, guild_id).await,
        _ => {
            error!("Unknown button interaction received: {}", button_id);
            error_embed("Unknown command!")
        }
    };

    respond_to_component(interaction, &ctx.http, embed).await;

    Ok(())
}

/// Replies to the errors users can hit in the bot's own style and logs the rest
pub async fn on_error(error: FrameworkError<'_, Data, Error>) {
    let (ctx, message) = match error {
        FrameworkError::GuildOnly { ctx, .. } => (ctx, "Poor Jimmy only works in servers!"),
        FrameworkError::NotAnOwner { ctx, .. } => {
            (ctx, "Only Poor Jimmy's owner can update its dependencies!")
        }
        FrameworkError::Command { error, ctx, .. } => {
            error!(
                "Error in command '{}': {}",
                ctx.command().qualified_name,
                error
            );
            return;
        }
        other => {
            if let Err(err) = poise::builtins::on_error(other).await {
                error!("Error while handling error: {}", err);
            }
            return;
        }
    };

    debug!(
        "Refused command '{}' from user {}: {}",
        ctx.command().qualified_name,
        ctx.author().name,
        message
    );

    if let Err(err) = ctx
        .send(CreateReply::default().embed(error_embed(message)))
        .await
    {
        error!("Failed to send error response: {}", err);
    }
}
