use tracing::{error, info};

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Remove Poor Jimmy from the voice channel
#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild_only command");

    ctx.data().guilds.cancel_disconnect_timer(guild_id);

    let manager = get_manager(ctx.serenity_context()).await;

    match manager.remove(guild_id).await {
        Ok(_) => {
            info!("Successfully left voice channel in guild {}", guild_id);
            reply(ctx, ok_embed("Poor Jimmy **left** the voice channel!")).await
        }
        Err(err) => {
            error!(
                "Failed to leave voice channel in guild {}: {}",
                guild_id, err
            );
            reply(
                ctx,
                error_embed(
                    "Error leaving voice channel! Ensure Poor Jimmy is in a voice channel with **/join**",
                ),
            )
            .await
        }
    }
}
