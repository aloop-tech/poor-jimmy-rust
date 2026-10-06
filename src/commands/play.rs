use songbird::input::{Input, YoutubeDl};
use tracing::warn;

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::{TrackMetadata, enqueue, load_metadata, voice_target},
        ytdlp::{MAX_PLAYLIST_TRACKS, UrlInfo, inspect_url},
    },
};

/// Play a song or playlist by title or URL, joining your voice channel if needed
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "A song title to search YouTube for, or a song/playlist URL"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");
    let serenity_ctx = ctx.serenity_context();
    let http_client = ctx.data().http_client.clone();

    // Check where the songs would play before any slow yt-dlp calls
    let target = match voice_target(serenity_ctx, guild_id, ctx.author().id).await {
        Ok(target) => target,
        Err(message) => return reply(ctx, error_embed(message)).await,
    };

    let (tracks, response): (Vec<(Input, TrackMetadata)>, String) = if is_url(&query) {
        // URLs go straight to yt-dlp, which decides whether it can play them
        match inspect_url(query.trim()).await {
            Err(err) => {
                warn!("Failed to load URL '{}': {}", query, err);
                return reply(
                    ctx,
                    error_embed(
                        "Couldn't load that track! Check the link or try a different search.",
                    ),
                )
                .await;
            }
            Ok(UrlInfo::Video(metadata)) => {
                let response = format!("**Queued** {}!", metadata.title);
                let source = YoutubeDl::new(http_client, query.trim().to_string());
                (vec![(source.into(), metadata)], response)
            }
            Ok(UrlInfo::Playlist(playlist)) => {
                if playlist.tracks.is_empty() {
                    return reply(ctx, error_embed("That playlist has no playable songs!")).await;
                }

                let mut response = format!("**Queued** {} songs", playlist.tracks.len());
                if let Some(title) = &playlist.title {
                    response.push_str(&format!(" from **{}**", title));
                }
                response.push('!');
                if playlist.truncated {
                    response.push_str(&format!(
                        "\nOnly the first {} songs are queued.",
                        MAX_PLAYLIST_TRACKS
                    ));
                }
                if playlist.unavailable > 0 {
                    response.push_str(&format!(
                        "\nSkipped {} unavailable song(s).",
                        playlist.unavailable
                    ));
                }

                let tracks = playlist
                    .tracks
                    .into_iter()
                    .map(|(url, metadata)| {
                        let source: Input = YoutubeDl::new(http_client.clone(), url).into();
                        (source, metadata)
                    })
                    .collect();

                (tracks, response)
            }
        }
    } else {
        // Anything else is a YouTube search
        let mut source: Input = YoutubeDl::new_search(http_client, query).into();
        let metadata = match load_metadata(&mut source).await {
            Ok(metadata) => metadata,
            Err(message) => return reply(ctx, error_embed(message)).await,
        };
        let response = format!("**Queued** {}!", metadata.title);
        (vec![(source, metadata)], response)
    };

    match enqueue(serenity_ctx, ctx.data(), guild_id, target, tracks).await {
        Ok(()) => reply(ctx, ok_embed(response)).await,
        Err(message) => reply(ctx, error_embed(message)).await,
    }
}

fn is_url(query: &str) -> bool {
    let query = query.trim();
    query.starts_with("https://") || query.starts_with("http://")
}

#[cfg(test)]
mod tests {
    use super::is_url;

    #[test]
    fn it_treats_http_links_as_urls() {
        assert!(is_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(is_url("https://youtu.be/e7qtC_e8Jxc?si=mtCnq8iVc253P89M"));
        assert!(is_url("http://soundcloud.com/artist/track"));
        assert!(is_url("  https://music.youtube.com/watch?v=abc  "));
    }

    #[test]
    fn it_treats_everything_else_as_a_search() {
        assert!(!is_url("never gonna give you up"));
        assert!(!is_url("youtube.com/watch?v=abc"));
        assert!(!is_url("-lofi beats"));
        assert!(!is_url("songs about https"));
        assert!(!is_url(""));
    }
}
