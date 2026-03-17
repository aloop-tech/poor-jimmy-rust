use serenity::all::{Color, CommandInteraction, ComponentInteraction, Context, CreateEmbed};
use songbird::{input::Input, tracks::Track};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

use crate::{
    handlers::track_play::TrackPlayHandler,
    utils::response::{respond_to_followup, respond_to_followup_component},
};

#[derive(Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub duration: Option<Duration>,
}

pub async fn enqueue_track(ctx: &Context, command: &CommandInteraction, mut source: Input) {
    let mut response_embed = CreateEmbed::default();

    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    if let Some(call) = manager.get(guild_id) {
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
            .clone()
            .unwrap_or_else(|| String::from("Unknown Track Title"));
        let track_thumbnail = metadata.thumbnail.clone();
        let track_duration = metadata.duration.clone();

        info!("Enqueueing track: '{}' in guild {}", track_title, guild_id);

        let custom_metadata = Arc::new(TrackMetadata {
            title: track_title.clone(),
            thumbnail_url: track_thumbnail.clone(),
            duration: track_duration,
        });

        let track_with_data = Track::new_with_data(source, custom_metadata);

        // Lock only for the enqueue operation, then release immediately.
        let track = {
            let mut handler = call.lock().await;
            handler.enqueue(track_with_data).await
        };

        let _ = track.add_event(
            songbird::Event::Track(songbird::TrackEvent::Playable),
            TrackPlayHandler {
                channel_id: command.channel_id,
                http: ctx.http.clone(),
                title: track_title.clone(),
                thumbnail: track_thumbnail.clone().unwrap_or_default(),
            },
        );

        let response_description = format!("**Queued** {}!", track_title);

        response_embed = response_embed
            .description(response_description)
            .color(Color::DARK_GREEN);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
    } else {
        error!(
            "Bot is not in a voice channel in guild {}. Cannot enqueue track.",
            guild_id
        );

        response_embed = response_embed
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
    }
}

pub async fn enqueue_track_list(ctx: &Context, command: &CommandInteraction, mut source: Input) {
    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    if let Some(call) = manager.get(guild_id) {
        let mut handler = call.lock().await;

        // Get metadata from the source
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
            .clone()
            .unwrap_or_else(|| String::from("Unknown Track Title"));
        let track_thumbnail = metadata.thumbnail.clone();
        let track_duration = metadata.duration.clone();

        info!("Enqueueing track: '{}' in guild {}", track_title, guild_id);

        // Create custom metadata to attach to the track
        let custom_metadata = Arc::new(TrackMetadata {
            title: track_title.clone(),
            thumbnail_url: track_thumbnail.clone(),
            duration: track_duration,
        });

        // Create track with attached metadata
        let track_with_data = Track::new_with_data(source, custom_metadata);

        // Play/enqueue song
        let track = handler.enqueue(track_with_data).await;

        let _ = track.add_event(
            songbird::Event::Track(songbird::TrackEvent::Playable),
            TrackPlayHandler {
                channel_id: command.channel_id,
                http: ctx.http.clone(),
                title: track_title.clone(),
                thumbnail: track_thumbnail.clone().unwrap_or_default(),
            },
        );
    } else {
        error!(
            "Bot is not in a voice channel in guild {}. Cannot enqueue track.",
            guild_id
        );
    }
}

/// Enqueue a track from a ComponentInteraction (e.g., button click).
/// This is similar to enqueue_track but works with ComponentInteraction instead of CommandInteraction.
pub async fn enqueue_track_component(
    ctx: &Context,
    interaction: &ComponentInteraction,
    mut source: Input,
) {
    let mut response_embed = CreateEmbed::default();

    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => {
            response_embed = response_embed
                .description("This command can only be used in a server!")
                .color(Color::DARK_RED);

            respond_to_followup_component(interaction, &ctx.http, response_embed, false).await;
            return;
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    if let Some(call) = manager.get(guild_id) {
        // Fetch metadata BEFORE locking the call handler.
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
            .clone()
            .unwrap_or_else(|| String::from("Unknown Track Title"));
        let track_thumbnail = metadata.thumbnail.clone();
        let track_duration = metadata.duration.clone();

        info!("Enqueueing track: '{}' in guild {}", track_title, guild_id);

        let custom_metadata = Arc::new(TrackMetadata {
            title: track_title.clone(),
            thumbnail_url: track_thumbnail.clone(),
            duration: track_duration,
        });

        let track_with_data = Track::new_with_data(source, custom_metadata);

        // Lock only for the enqueue operation, then release immediately.
        let track = {
            let mut handler = call.lock().await;
            handler.enqueue(track_with_data).await
        };

        let _ = track.add_event(
            songbird::Event::Track(songbird::TrackEvent::Playable),
            TrackPlayHandler {
                channel_id: interaction.channel_id,
                http: ctx.http.clone(),
                title: track_title.clone(),
                thumbnail: track_thumbnail.clone().unwrap_or_default(),
            },
        );

        let response_description = format!("**Queued** {}!", track_title);

        response_embed = response_embed
            .description(response_description)
            .color(Color::DARK_GREEN);

        respond_to_followup_component(interaction, &ctx.http, response_embed, false).await;
    } else {
        error!(
            "Bot is not in a voice channel in guild {}. Cannot enqueue track.",
            guild_id
        );

        response_embed = response_embed
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);

        respond_to_followup_component(interaction, &ctx.http, response_embed, false).await;
    }
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
