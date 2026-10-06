use songbird::{Event, TrackEvent};
use tracing::{error, info, warn};

use crate::{
    data::{Context, Error},
    handlers::track_end::TrackEndNotifier,
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Summon Poor Jimmy to your voice channel
#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");
    let user_id = ctx.author().id;
    let serenity_ctx = ctx.serenity_context();

    // Extract voice channel ID from cache, ensuring guild reference is dropped
    let voice_channel_id = {
        serenity_ctx.cache.guild(guild_id).and_then(|g| {
            g.voice_states
                .get(&user_id)
                .and_then(|voice_state| voice_state.channel_id)
        })
    };

    // Check if we successfully got the guild from cache
    if voice_channel_id.is_none() && serenity_ctx.cache.guild(guild_id).is_none() {
        error!("Failed to find guild {} in cache", guild_id);
        return reply(ctx, error_embed("Error joining voice channel")).await;
    }

    let Some(connect_to) = voice_channel_id else {
        warn!(
            "User {} attempted to use /join but is not in a voice channel (guild {})",
            user_id, guild_id
        );
        return reply(ctx, error_embed("You're not in a voice channel!")).await;
    };

    let manager = get_manager(serenity_ctx).await;

    info!(
        "Attempting to join voice channel {} in guild {}",
        connect_to, guild_id
    );

    match manager.join(guild_id, connect_to).await {
        Ok(call) => {
            {
                let mut handler = call.lock().await;

                handler.remove_all_global_events();

                handler.add_global_event(
                    Event::Track(TrackEvent::End),
                    TrackEndNotifier {
                        channel_id: ctx.channel_id(),
                        http: serenity_ctx.http.clone(),
                        call: call.clone(),
                        guild_id,
                        manager: manager.clone(),
                        disconnect_timers: ctx.data().disconnect_timers.clone(),
                        timeout_minutes: ctx.data().auto_disconnect_minutes,
                    },
                );
            } // lock released before any HTTP requests

            info!(
                "Successfully joined voice channel {} in guild {}",
                connect_to, guild_id
            );

            reply(ctx, ok_embed("Poor Jimmy **joined** the voice channel!")).await
        }
        Err(err) => {
            error!(
                "Failed to join voice channel {} in guild {}: {}",
                connect_to, guild_id, err
            );
            reply(ctx, error_embed("Error joining voice channel!")).await
        }
    }
}
