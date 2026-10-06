use serenity::all::{Context as SerenityContext, CreateEmbed, GuildId};

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Stop the current song and clear the queue
#[poise::command(slash_command, guild_only)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().expect("guild_only command");
    reply(ctx, action(ctx.serenity_context(), guild_id).await).await
}

/// Stops playback and clears the queue. Shared by /clear and the Clear button.
pub async fn action(ctx: &SerenityContext, guild_id: GuildId) -> CreateEmbed {
    let Some(call) = get_manager(ctx).await.get(guild_id) else {
        return error_embed(
            "Error clearing queue! Ensure Poor Jimmy is in a voice channel with **/join**",
        );
    };

    let queue_length = {
        let handler = call.lock().await;
        let queue_length = handler.queue().len();

        if queue_length > 0 {
            handler.queue().stop();
        }

        queue_length
    }; // Release lock on handler

    if queue_length == 0 {
        ok_embed("There is nothing to clear!")
    } else {
        ok_embed("Queue **cleared!**")
    }
}
