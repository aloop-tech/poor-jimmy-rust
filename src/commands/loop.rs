use serenity::all::{Context as SerenityContext, CreateEmbed, GuildId};
use songbird::tracks::LoopState;
use tracing::error;

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Enable/disable looping for the current song
#[poise::command(slash_command, guild_only, rename = "loop")]
pub async fn loop_song(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().expect("guild_only command");
    reply(ctx, action(ctx.serenity_context(), guild_id).await).await
}

/// Toggles looping on the current song. Shared by /loop and the Loop button.
pub async fn action(ctx: &SerenityContext, guild_id: GuildId) -> CreateEmbed {
    let Some(call) = get_manager(ctx).await.get(guild_id) else {
        return error_embed(
            "Error looping song! Ensure Poor Jimmy is in a voice channel with **/join**",
        );
    };

    let current_song = {
        let handler = call.lock().await;
        handler.queue().current()
    }; // Release lock on handler

    let Some(song) = current_song else {
        return ok_embed("There is no song to loop!");
    };

    let is_looping = match song.get_info().await {
        Ok(state) => state.loops.eq(&LoopState::Infinite),
        Err(why) => {
            error!("Error getting song state: {why}");
            return error_embed("Error looping song!");
        }
    };

    if is_looping {
        match song.disable_loop() {
            Ok(_) => ok_embed("Disabled **looping!**"),
            Err(why) => {
                error!("Error disabling looping: {why}");
                error_embed("Error looping song!")
            }
        }
    } else {
        match song.enable_loop() {
            Ok(_) => {
                ok_embed("Enabled **looping!** Use **/loop** again to disable or **/skip** to skip")
            }
            Err(why) => {
                error!("Error looping song: {why}");
                error_embed("Error looping song!")
            }
        }
    }
}
