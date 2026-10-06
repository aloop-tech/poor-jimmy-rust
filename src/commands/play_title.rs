use songbird::input::YoutubeDl;

use crate::{
    data::{Context, Error},
    utils::{response::reply, track_utils::enqueue},
};

/// Play the audio from a Youtube video searching by title
#[poise::command(slash_command, guild_only, rename = "play-title")]
pub async fn play_title(
    ctx: Context<'_>,
    #[description = "A Youtube video title"] title: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");

    // Get the audio source for the search
    let source = YoutubeDl::new_search(ctx.data().http_client.clone(), title);

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
