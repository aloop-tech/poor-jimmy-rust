use tracing::info;

use crate::{
    data::{Context, Error},
    utils::{
        queue::remove_upcoming,
        response::{error_embed, ok_embed, reply},
        track_utils::{TrackMetadata, get_manager},
    },
};

/// Remove a song from the queue by its /list position
#[poise::command(slash_command, guild_only)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The song's position in /list"]
    #[min = 1]
    position: usize,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");

    let Some(call) = get_manager(ctx.serenity_context()).await.get(guild_id) else {
        return reply(
            ctx,
            error_embed(
                "Error removing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            ),
        )
        .await;
    };

    let removed = {
        let handler = call.lock().await;
        handler
            .queue()
            .modify_queue(|queue| remove_upcoming(queue, position))
    }; // Release lock on handler

    match removed {
        Ok(queued) => {
            // Queued tracks are already loaded (paused) in the mixer, so stop
            // the removed one to free it
            let handle = queued.handle();
            let _ = handle.stop();
            let title = handle.data::<TrackMetadata>().title.clone();

            info!("Removed '{}' from the queue in guild {}", title, guild_id);
            reply(
                ctx,
                ok_embed(format!("**Removed** {} from the queue!", title)),
            )
            .await
        }
        Err(err) => reply(ctx, error_embed(err.message())).await,
    }
}
