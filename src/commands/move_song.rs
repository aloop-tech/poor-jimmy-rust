use crate::{
    data::{Context, Error},
    utils::{
        queue::move_upcoming,
        response::{error_embed, ok_embed, reply},
        track_utils::{TrackMetadata, get_manager},
    },
};

/// Move a song to a different spot in the queue
#[poise::command(slash_command, guild_only, rename = "move")]
pub async fn move_song(
    ctx: Context<'_>,
    #[description = "The song's position in /list"]
    #[min = 2]
    from: usize,
    #[description = "Where it should go"]
    #[min = 2]
    to: usize,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");

    let Some(call) = get_manager(ctx.serenity_context()).await.get(guild_id) else {
        return reply(
            ctx,
            error_embed(
                "Error moving song! Ensure Poor Jimmy is in a voice channel with **/join**",
            ),
        )
        .await;
    };

    let moved = {
        let handler = call.lock().await;
        handler.queue().modify_queue(|queue| {
            move_upcoming(queue, from, to)
                .map(|()| queue[to - 1].handle().data::<TrackMetadata>().title.clone())
        })
    }; // Release lock on handler

    match moved {
        Ok(title) => {
            reply(
                ctx,
                ok_embed(format!("**Moved** {} to position {}!", title, to)),
            )
            .await
        }
        Err(err) => reply(ctx, error_embed(err.message())).await,
    }
}
