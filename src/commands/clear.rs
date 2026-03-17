use serenity::{
    all::{Color, CommandInteraction, ComponentInteraction, CreateEmbed},
    client::Context,
};
use tracing::error;

use crate::utils::response::{respond_to_button, respond_to_error_button, respond_to_followup};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer clear command: {}", err);
        return;
    }

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        let queue_length = handler.queue().len();

        if queue_length == 0 {
            let embed = CreateEmbed::new()
                .description("There is nothing to clear!")
                .color(Color::DARK_GREEN);
            respond_to_followup(command, &ctx.http, embed, false).await;
        } else {
            handler.queue().stop();

            let embed = CreateEmbed::new()
                .description("Queue **cleared!**")
                .color(Color::DARK_GREEN);
            respond_to_followup(command, &ctx.http, embed, false).await;
        }
    } else {
        let embed = CreateEmbed::new()
            .description(
                "Error clearing queue! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);
        respond_to_followup(command, &ctx.http, embed, false).await;
    }
}

pub async fn handle_button(ctx: &Context, command: &ComponentInteraction) {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        let queue_length = handler.queue().len();

        if queue_length == 0 {
            respond_to_button(
                command,
                &ctx.http,
                format!("There is nothing to clear!"),
                false,
            )
            .await;
        } else {
            handler.queue().stop();

            respond_to_button(command, &ctx.http, format!("Queue **cleared!**"), false).await;
        }
    } else {
        respond_to_error_button(
            command,
            &ctx.http,
            format!("Error clearing queue! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("clear")
        .description("Stop the current song and clear the queue")
}
