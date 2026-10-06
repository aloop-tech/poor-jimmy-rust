use serenity::all::{Context as SerenityContext, CreateEmbed, GuildId, UserId};
use songbird::{Songbird, input::Input, tracks::Track};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

use crate::{
    data::Data,
    handlers::track_play::TrackPlayHandler,
    utils::{
        response::{error_embed, ok_embed},
        voice::{connect, user_voice_channel},
    },
};

#[derive(Clone)]
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

/// Fetches track metadata, joins the requesting user's voice channel if the bot
/// isn't in one yet, enqueues the source, and registers the playback
/// notification handler. Returns an embed describing the result (success or
/// error) for the caller to send.
pub async fn enqueue(
    ctx: &SerenityContext,
    data: &Data,
    guild_id: GuildId,
    user_id: UserId,
    mut source: Input,
) -> CreateEmbed {
    let manager = get_manager(ctx).await;
    let existing_call = manager.get(guild_id);

    // Check where to join before the slow metadata fetch
    let join_channel = match &existing_call {
        Some(_) => None,
        None => match user_voice_channel(&ctx.cache, guild_id, user_id) {
            Some(channel_id) => Some(channel_id),
            None => return error_embed("Join a voice channel first, then try again!"),
        },
    };

    // Fetch metadata BEFORE locking the call handler — aux_metadata spawns yt-dlp
    // and can take several seconds. Holding the call lock during that time blocks
    // songbird's event dispatch and prevents audio from playing. A failure here
    // means yt-dlp couldn't load the track, so don't queue (or join) for it.
    debug!("Fetching track metadata for guild {}", guild_id);
    let metadata = match source.aux_metadata().await {
        Ok(meta) => meta,
        Err(err) => {
            warn!("Failed to fetch track metadata: {}", err);
            return error_embed(
                "Couldn't load that track! Check the link or try a different search.",
            );
        }
    };

    let call = match (existing_call, join_channel) {
        (Some(call), _) => call,
        (None, Some(channel_id)) => {
            match connect(ctx, data, manager.clone(), guild_id, channel_id).await {
                Ok(call) => call,
                Err(err) => {
                    error!(
                        "Failed to join voice channel {} in guild {}: {}",
                        channel_id, guild_id, err
                    );
                    return error_embed("Error joining voice channel!");
                }
            }
        }
        (None, None) => unreachable!("join_channel is set whenever there's no call"),
    };

    let track_title = metadata
        .title
        .unwrap_or_else(|| String::from("Unknown Track Title"));
    let track_thumbnail = metadata.thumbnail;
    let track_duration = metadata.duration;

    info!("Enqueueing track: '{}' in guild {}", track_title, guild_id);

    let custom_metadata = Arc::new(TrackMetadata {
        title: track_title.clone(),
        thumbnail_url: track_thumbnail.clone(),
        duration: track_duration,
    });

    let track_with_data = Track::new_with_data(source, custom_metadata);

    // Cancel any pending disconnect timer since we're adding a track
    data.guilds.cancel_idle_timer(guild_id);

    // Lock only for the enqueue operation, then release immediately.
    let track = {
        let mut handler = call.lock().await;
        handler.enqueue(track_with_data).await
    };

    let _ = track.add_event(
        songbird::Event::Track(songbird::TrackEvent::Playable),
        TrackPlayHandler {
            http: ctx.http.clone(),
            guilds: data.guilds.clone(),
            guild_id,
            title: track_title.clone(),
            thumbnail: track_thumbnail.unwrap_or_default(),
        },
    );

    ok_embed(format!("**Queued** {}!", track_title))
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
