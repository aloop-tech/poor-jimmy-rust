use tracing::{error, warn};

use crate::{
    data::{Context, Error},
    utils::{
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
        voice::{connect, user_voice_channel},
    },
};

/// Summon Poor Jimmy to your voice channel
#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");
    let user_id = ctx.author().id;
    let serenity_ctx = ctx.serenity_context();

    let voice_channel_id = user_voice_channel(&serenity_ctx.cache, guild_id, user_id);

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

    match connect(serenity_ctx, ctx.data(), manager, guild_id, connect_to).await {
        Ok(_) => reply(ctx, ok_embed("Poor Jimmy **joined** the voice channel!")).await,
        Err(err) => {
            error!(
                "Failed to join voice channel {} in guild {}: {}",
                connect_to, guild_id, err
            );
            reply(ctx, error_embed("Error joining voice channel!")).await
        }
    }
}
