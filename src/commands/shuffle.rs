use crate::{
    data::{Context, Error},
    utils::{
        queue::shuffle_upcoming,
        response::{error_embed, ok_embed, reply},
        track_utils::get_manager,
    },
};

/// Shuffle the upcoming songs in the queue
#[poise::command(slash_command, guild_only)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().expect("guild_only command");

    let Some(call) = get_manager(ctx.serenity_context()).await.get(guild_id) else {
        return reply(
            ctx,
            error_embed(
                "Error shuffling queue! Ensure Poor Jimmy is in a voice channel with **/join**",
            ),
        )
        .await;
    };

    let shuffled = {
        let handler = call.lock().await;
        handler
            .queue()
            .modify_queue(|queue| shuffle_upcoming(queue, &mut fastrand::Rng::new()))
    }; // Release lock on handler

    if shuffled < 2 {
        return reply(
            ctx,
            ok_embed("There aren't enough upcoming songs to shuffle!"),
        )
        .await;
    }

    reply(
        ctx,
        ok_embed(format!("**Shuffled** {} upcoming songs!", shuffled)),
    )
    .await
}
