use tracing::info;

use crate::{
    data::{Context, Error},
    utils::response::{ok_embed, reply},
};

/// Respond with Pong!
#[poise::command(slash_command, guild_only)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild_only command");

    info!("Ping! From guild id: {guild_id}");

    reply(ctx, ok_embed("Pong!")).await
}
