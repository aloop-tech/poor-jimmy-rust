use songbird::input::YoutubeDl;

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, reply},
        track_utils::enqueue,
    },
};

/// Play the audio from a Youtube video URL
#[poise::command(slash_command, guild_only, rename = "play-url")]
pub async fn play_url(
    ctx: Context<'_>,
    #[description = "A Youtube video URL"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Validate its a valid Youtube URL
    if !is_valid_youtube_url(&url) {
        return reply(
            ctx,
            error_embed("Please provide a valid **/watch** Youtube URL"),
        )
        .await;
    }

    let guild_id = ctx.guild_id().expect("guild_only command");

    // Get the audio source for the URL
    let source = YoutubeDl::new(ctx.data().http_client.clone(), url);

    let embed = enqueue(
        ctx.serenity_context(),
        ctx.data(),
        guild_id,
        ctx.channel_id(),
        source.into(),
    )
    .await;

    reply(ctx, embed).await
}

fn is_valid_youtube_url(url: &String) -> bool {
    (url.contains("youtube.com") && (url.contains("/watch"))) || url.contains("youtu.be")
}

#[cfg(test)]
mod tests {
    use crate::commands::play_url::is_valid_youtube_url;

    #[test]
    fn it_validates_valid_youtube_urls() {
        let valid_watch_url = String::from("https://www.youtube.com/watch?id=12345");
        let valid_share_url = String::from("https://youtu.be/e7qtC_e8Jxc?si=mtCnq8iVc253P89M");

        assert_eq!(true, is_valid_youtube_url(&valid_watch_url));
        assert_eq!(true, is_valid_youtube_url(&valid_share_url));
    }

    #[test]
    fn it_validates_invalid_youtube_urls() {
        let invalid_url = String::from("https://www.you.tube.com/watch?id=12345");
        let another_invalid_url =
            String::from("https://www.youtube.com/results?search_query=title");

        assert_eq!(false, is_valid_youtube_url(&invalid_url));
        assert_eq!(false, is_valid_youtube_url(&another_invalid_url));
    }

    #[test]
    fn it_validates_youtube_url_edge_cases() {
        // Empty string
        assert_eq!(false, is_valid_youtube_url(&String::from("")));

        // Non-YouTube URLs
        assert_eq!(false, is_valid_youtube_url(&String::from("https://vimeo.com/12345")));
        assert_eq!(false, is_valid_youtube_url(&String::from("https://www.google.com")));

        // YouTube URLs without watch or youtu.be
        assert_eq!(false, is_valid_youtube_url(&String::from("https://www.youtube.com/")));
        assert_eq!(false, is_valid_youtube_url(&String::from("https://www.youtube.com/channel/test")));

        // Valid variations
        assert_eq!(true, is_valid_youtube_url(&String::from("https://youtube.com/watch?v=abc123")));
        assert_eq!(true, is_valid_youtube_url(&String::from("http://www.youtube.com/watch?v=test")));
    }

    #[test]
    fn it_validates_youtube_mobile_urls() {
        let mobile_url = String::from("https://m.youtube.com/watch?v=12345");
        assert_eq!(true, is_valid_youtube_url(&mobile_url));
    }
}
