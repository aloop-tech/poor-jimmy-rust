use serenity::all::{
    ChannelId, Color, CommandInteraction, ComponentInteraction, Context, CreateEmbed, GuildId,
};
use songbird::{input::Input, tracks::Track};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

use crate::{
    handlers::track_play::TrackPlayHandler,
    utils::{
        response::{respond_to_followup, respond_to_followup_component},
        type_map::{cancel_disconnect_timer, get_disconnect_timers},
    },
};

#[derive(Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub duration: Option<Duration>,
}

pub async fn enqueue_track(ctx: &Context, command: &CommandInteraction, source: Input) {
    let guild_id = command.guild_id.unwrap();
    let embed = do_enqueue(ctx, guild_id, command.channel_id, source).await;
    respond_to_followup(command, &ctx.http, embed, false).await;
}

pub async fn enqueue_track_component(
    ctx: &Context,
    interaction: &ComponentInteraction,
    source: Input,
) {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => {
            let embed = CreateEmbed::default()
                .description("This command can only be used in a server!")
                .color(Color::DARK_RED);
            respond_to_followup_component(interaction, &ctx.http, embed, false).await;
            return;
        }
    };
    let embed = do_enqueue(ctx, guild_id, interaction.channel_id, source).await;
    respond_to_followup_component(interaction, &ctx.http, embed, false).await;
}

/// Fetches track metadata, enqueues the source into the active voice call, and
/// registers the playback notification handler. Returns an embed describing the
/// result (success or error) for the caller to send.
async fn do_enqueue(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
    mut source: Input,
) -> CreateEmbed {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let Some(call) = manager.get(guild_id) else {
        error!(
            "Bot is not in a voice channel in guild {}. Cannot enqueue track.",
            guild_id
        );
        return CreateEmbed::default()
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);
    };

    // Fetch metadata BEFORE locking the call handler — aux_metadata spawns yt-dlp
    // and can take several seconds. Holding the call lock during that time blocks
    // songbird's event dispatch and prevents audio from playing.
    debug!("Fetching track metadata for guild {}", guild_id);
    let metadata = match source.aux_metadata().await {
        Ok(meta) => meta,
        Err(err) => {
            warn!("Failed to fetch track metadata: {}. Using defaults.", err);
            Default::default()
        }
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
    let disconnect_timers = get_disconnect_timers(ctx).await;
    cancel_disconnect_timer(&disconnect_timers, guild_id);

    // Lock only for the enqueue operation, then release immediately.
    let track = {
        let mut handler = call.lock().await;
        handler.enqueue(track_with_data).await
    };

    let _ = track.add_event(
        songbird::Event::Track(songbird::TrackEvent::Playable),
        TrackPlayHandler {
            channel_id,
            http: ctx.http.clone(),
            title: track_title.clone(),
            thumbnail: track_thumbnail.unwrap_or_default(),
        },
    );

    CreateEmbed::default()
        .description(format!("**Queued** {}!", track_title))
        .color(Color::DARK_GREEN)
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
