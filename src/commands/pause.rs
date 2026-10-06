use serenity::all::{Context as SerenityContext, CreateEmbed, GuildId};
use songbird::tracks::PlayMode;
use tracing::error;

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Pause the currently playing song
#[poise::command(slash_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().expect("guild_only command");
    reply(ctx, action(ctx.serenity_context(), guild_id).await).await
}

/// Pauses the current song if it's playing. Shared by /pause and the Pause button.
pub async fn action(ctx: &SerenityContext, guild_id: GuildId) -> CreateEmbed {
    let Some(call) = get_manager(ctx).await.get(guild_id) else {
        return error_embed(
            "Error pausing song! Ensure Poor Jimmy is in a voice channel with **/join**",
        );
    };

    let current_song = {
        let handler = call.lock().await;
        handler.queue().current()
    }; // Release lock on handler

    let Some(song) = current_song else {
        return ok_embed("There is no song to pause!");
    };

    let song_state = match song.get_info().await {
        Ok(state) => state.playing,
        Err(why) => {
            error!("Error getting song state: {why}");
            return error_embed("Error pausing song!");
        }
    };

    if !matches!(song_state, PlayMode::Play) {
        return ok_embed("The song is currently paused!");
    }

    match song.pause() {
        Ok(_) => ok_embed("Song **paused!** Use **/resume** to continue playback"),
        Err(why) => {
            error!("Error pausing song: {why}");
            error_embed("Error pausing song!")
        }
    }
}
