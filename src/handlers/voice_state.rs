use serenity::all::{Context as SerenityContext, CreateMessage, GuildId, VoiceState};
use songbird::tracks::PlayMode;
use tracing::{error, info};

use crate::{
    data::Data,
    utils::{
        response::{announce, ok_embed},
        track_utils::get_manager,
        voice::{VoiceHandles, has_listeners},
    },
};

/// Pause and start the disconnect timer when everyone else leaves the bot's
/// voice channel; resume when someone comes back before it fires.
pub async fn handle_voice_state_update(ctx: &SerenityContext, data: &Data, new: &VoiceState) {
    let Some(guild_id) = new.guild_id else {
        return;
    };

    let manager = get_manager(ctx).await;

    // Only guilds where the bot is connected
    if manager.get(guild_id).is_none() {
        return;
    }

    // None while the cache doesn't show the bot in a channel yet (mid-join)
    let Some(listeners) = has_listeners(&ctx.cache, guild_id) else {
        return;
    };

    let waiting = data
        .guilds
        .with(guild_id, |state| state.waiting_for_listeners);

    let voice = VoiceHandles::new(ctx, manager, data);

    match (listeners, waiting) {
        (false, false) => channel_emptied(&voice, guild_id).await,
        (true, true) => listeners_returned(&voice, guild_id).await,
        // No change
        _ => {}
    }
}

async fn channel_emptied(voice: &VoiceHandles, guild_id: GuildId) {
    info!("Voice channel emptied in guild {}", guild_id);

    let paused = pause_current_track(voice, guild_id).await;

    let timer_running = voice.guilds.with(guild_id, |state| {
        state.waiting_for_listeners = true;
        state.paused_for_empty_channel = paused;
        state.disconnect_timer.is_some()
    });

    // Nothing was playing and a timer (e.g. queue ended) is already counting down
    if !paused && timer_running {
        return;
    }

    if !timer_running {
        voice.start_disconnect_timer(guild_id);
    }

    let description = if paused {
        format!(
            "Everyone left, so playback is **paused**. Poor Jimmy will leave in {} minutes unless someone comes back.",
            voice.timeout_minutes
        )
    } else {
        format!(
            "Everyone left. Poor Jimmy will leave in {} minutes unless someone comes back.",
            voice.timeout_minutes
        )
    };

    announce(
        &voice.http,
        &voice.guilds,
        guild_id,
        CreateMessage::new().embed(ok_embed(description)),
    )
    .await;
}

async fn listeners_returned(voice: &VoiceHandles, guild_id: GuildId) {
    info!("Listeners returned in guild {}", guild_id);

    let paused_for_empty = voice.guilds.with(guild_id, |state| {
        state.waiting_for_listeners = false;
        std::mem::take(&mut state.paused_for_empty_channel)
    });

    let Some(call) = voice.manager.get(guild_id) else {
        return;
    };

    let (current, queue_is_empty) = {
        let handler = call.lock().await;
        (handler.queue().current(), handler.queue().is_empty())
    };

    // The timer was only for the empty channel; if the queue ended meanwhile,
    // its own timer should keep running
    if !queue_is_empty {
        voice.guilds.cancel_disconnect_timer(guild_id);
    }

    if !paused_for_empty {
        return;
    }

    let Some(track) = current else {
        return;
    };

    if let Err(err) = track.play() {
        error!("Failed to resume after listeners returned: {}", err);
        return;
    }

    announce(
        &voice.http,
        &voice.guilds,
        guild_id,
        CreateMessage::new().embed(ok_embed("Welcome back! Playback **resumed!**")),
    )
    .await;
}

/// Pause the current track if it's playing. Returns whether it paused anything.
async fn pause_current_track(voice: &VoiceHandles, guild_id: GuildId) -> bool {
    let Some(call) = voice.manager.get(guild_id) else {
        return false;
    };

    let current = call.lock().await.queue().current();

    let Some(track) = current else {
        return false;
    };

    match track.get_info().await {
        Ok(info) if matches!(info.playing, PlayMode::Play) => match track.pause() {
            Ok(_) => true,
            Err(err) => {
                error!("Failed to pause for empty channel: {}", err);
                false
            }
        },
        Ok(_) => false,
        Err(err) => {
            error!("Failed to get track state: {}", err);
            false
        }
    }
}
