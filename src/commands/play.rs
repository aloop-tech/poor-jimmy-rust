use songbird::input::YoutubeDl;

use crate::{
    data::{Context, Error},
    utils::{response::reply, track_utils::enqueue},
};

/// Play a song by title or URL, joining your voice channel if needed
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "A song title to search YouTube for, or a URL"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");
    let http_client = ctx.data().http_client.clone();

    // URLs go straight to yt-dlp, which decides whether it can play them;
    // anything else is a YouTube search
    let source = if is_url(&query) {
        YoutubeDl::new(http_client, query)
    } else {
        YoutubeDl::new_search(http_client, query)
    };

    let embed = enqueue(
        ctx.serenity_context(),
        ctx.data(),
        guild_id,
        ctx.author().id,
        source.into(),
    )
    .await;

    reply(ctx, embed).await
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
