use serenity::{
    all::{ChannelId, Context as SerenityContext, GuildId, UserId},
    prelude::Mutex,
};
use songbird::{Call, Songbird, input::Input, tracks::Track};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

use crate::{
    data::Data,
    handlers::track_play::TrackPlayHandler,
    utils::voice::{connect, user_voice_channel},
};

#[derive(Clone, Debug)]
pub struct TrackMetadata {
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub duration: Option<Duration>,
}

/// The songbird voice manager registered on the client at startup
pub async fn get_manager(ctx: &SerenityContext) -> Arc<Songbird> {
    songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.")
}

// The functions below return user-facing error messages for the caller to show.

/// Where new tracks will play: the bot's current call, or the requesting
/// user's voice channel to join first
pub enum VoiceTarget {
    Connected(Arc<Mutex<Call>>),
    Join(ChannelId),
}

/// Work out where a user's tracks would play. Cheap, so callers check this
/// before any slow metadata fetching.
pub async fn voice_target(
    ctx: &SerenityContext,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<VoiceTarget, String> {
    if let Some(call) = get_manager(ctx).await.get(guild_id) {
        return Ok(VoiceTarget::Connected(call));
    }

    match user_voice_channel(&ctx.cache, guild_id, user_id) {
        Some(channel_id) => Ok(VoiceTarget::Join(channel_id)),
        None => Err("Join a voice channel first, then try again!".to_string()),
    }
}

/// Ask the source for its title, duration, and thumbnail. For yt-dlp sources this
/// runs yt-dlp (several seconds) and caches the result on the source; a failure
/// means yt-dlp couldn't load the track.
pub async fn load_metadata(source: &mut Input) -> Result<TrackMetadata, String> {
    match source.aux_metadata().await {
        Ok(meta) => Ok(TrackMetadata {
            title: meta
                .title
                .unwrap_or_else(|| String::from("Unknown Track Title")),
            thumbnail_url: meta.thumbnail,
            duration: meta.duration,
        }),
        Err(err) => {
            warn!("Failed to fetch track metadata: {}", err);
            Err("Couldn't load that track! Check the link or try a different search.".to_string())
        }
    }
}

/// Join the target voice channel if needed, then queue the tracks in order and
/// register their "Now playing" handlers. Metadata must already be loaded.
pub async fn enqueue(
    ctx: &SerenityContext,
    data: &Data,
    guild_id: GuildId,
    target: VoiceTarget,
    tracks: Vec<(Input, TrackMetadata)>,
) -> Result<(), String> {
    let call = match target {
        VoiceTarget::Connected(call) => call,
        VoiceTarget::Join(channel_id) => {
            let manager = get_manager(ctx).await;
            match connect(ctx, data, manager, guild_id, channel_id).await {
                Ok(call) => call,
                Err(err) => {
                    error!(
                        "Failed to join voice channel {} in guild {}: {}",
                        channel_id, guild_id, err
                    );
                    return Err("Error joining voice channel!".to_string());
                }
            }
        }
    };

    // Cancel any pending disconnect timer since we're adding tracks
    data.guilds.cancel_idle_timer(guild_id);

    for (source, metadata) in tracks {
        info!(
            "Enqueueing track: '{}' in guild {}",
            metadata.title, guild_id
        );

        // Start loading the next track 5 seconds before this one ends. Passing
        // this ourselves (instead of songbird's `enqueue`, which asks the source)
        // avoids a yt-dlp call per track while holding the call lock.
        let preload_time = metadata
            .duration
            .map(|duration| duration.saturating_sub(Duration::from_secs(5)));

        let play_handler = TrackPlayHandler {
            http: ctx.http.clone(),
            guilds: data.guilds.clone(),
            guild_id,
            title: metadata.title.clone(),
            thumbnail: metadata.thumbnail_url.clone().unwrap_or_default(),
        };

        let track = Track::new_with_data(source, Arc::new(metadata));

        // Lock only for the enqueue operation, then release immediately.
        let handle = {
            let mut handler = call.lock().await;
            handler.enqueue_with_preload(track, preload_time)
        };

        if let Err(err) = handle.add_event(
            songbird::Event::Track(songbird::TrackEvent::Playable),
            play_handler,
        ) {
            debug!("Track ended before its play handler was added: {}", err);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_metadata_creation() {
        let metadata = TrackMetadata {
            title: "Test Song".to_string(),
            thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
            duration: Some(Duration::from_secs(180)),
        };

        assert_eq!(metadata.title, "Test Song");
        assert_eq!(
            metadata.thumbnail_url,
            Some("https://example.com/thumb.jpg".to_string())
        );
        assert_eq!(metadata.duration, Some(Duration::from_secs(180)));
    }

    #[test]
    fn test_track_metadata_clone() {
        let metadata = TrackMetadata {
            title: "Original".to_string(),
            thumbnail_url: None,
            duration: None,
        };

        let cloned = metadata.clone();
        assert_eq!(cloned.title, "Original");
        assert_eq!(cloned.thumbnail_url, None);
        assert_eq!(cloned.duration, None);
    }

    #[test]
    fn test_track_metadata_with_no_thumbnail() {
        let metadata = TrackMetadata {
            title: "No Thumbnail Song".to_string(),
            thumbnail_url: None,
            duration: Some(Duration::from_secs(240)),
        };

        assert!(metadata.thumbnail_url.is_none());
    }

    #[test]
    fn test_track_metadata_with_no_duration() {
        let metadata = TrackMetadata {
            title: "Live Stream".to_string(),
            thumbnail_url: Some("https://example.com/live.jpg".to_string()),
            duration: None,
        };

        assert!(metadata.duration.is_none());
    }
}
