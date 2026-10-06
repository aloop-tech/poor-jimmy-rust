use serenity::all::{Context as SerenityContext, CreateEmbed, GuildId};
use tracing::{error, warn};

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Skip the currently playing song
#[poise::command(slash_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().expect("guild_only command");
    reply(ctx, action(ctx.serenity_context(), guild_id).await).await
}

/// Skips the current song. Shared by /skip and the Skip button.
pub async fn action(ctx: &SerenityContext, guild_id: GuildId) -> CreateEmbed {
    let Some(call) = get_manager(ctx).await.get(guild_id) else {
        warn!(
            "Attempted to skip song but bot is not in voice channel (guild {})",
            guild_id
        );
        return error_embed(
            "Error skipping song! Ensure Poor Jimmy is in a voice channel with **/join**",
        );
    };

    let current_song = {
        let handler = call.lock().await;
        handler.queue().current()
    }; // Release lock on handler

    let Some(song) = current_song else {
        return error_embed("There is no song currently playing!");
    };

    match song.stop() {
        Ok(_) => ok_embed("Song **skipped!**"),
        Err(why) => {
            error!("Error skipping track in guild {}: {}", guild_id, why);
            error_embed("Error skipping song!")
        }
    }
}
