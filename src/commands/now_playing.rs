use poise::CreateReply;
use tracing::{error, warn};

use crate::{
    components::music_buttons::create_music_buttons,
    data::{Context, Error},
    utils::{
        format::create_progress_bar,
        response::{error_embed, ok_embed, reply},
        track_utils::{TrackMetadata, get_manager},
    },
};

/// Show the currently playing song with progress
#[poise::command(slash_command, guild_only, rename = "now-playing")]
pub async fn now_playing(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");

    let Some(call) = get_manager(ctx.serenity_context()).await.get(guild_id) else {
        warn!(
            "Attempted to get now playing but bot is not in voice channel (guild {})",
            guild_id
        );
        return reply(
            ctx,
            error_embed("Error! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    };

    let current_track = {
        let handler = call.lock().await;
        handler.queue().current()
    }; // Release lock on handler

    let Some(current_track) = current_track else {
        warn!("No track currently playing in guild {}", guild_id);
        return reply(ctx, ok_embed("No song is currently playing!")).await;
    };

    // Get track metadata
    let metadata = current_track.data::<TrackMetadata>();
    let title = &metadata.title;

    // Get playback info
    let track_info = match current_track.get_info().await {
        Ok(info) => info,
        Err(err) => {
            error!("Failed to get track info in guild {}: {}", guild_id, err);
            return reply(ctx, error_embed("Error getting track information!")).await;
        }
    };

    // Format response with progress bar
    let progress_bar = create_progress_bar(track_info.position, metadata.duration, 20);

    let mut embed = ok_embed(format!("**Now Playing:**\n{}\n\n{}", title, progress_bar));

    if let Some(url) = &metadata.thumbnail_url {
        embed = embed.thumbnail(url);
    }

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(create_music_buttons()),
    )
    .await?;

    Ok(())
}
