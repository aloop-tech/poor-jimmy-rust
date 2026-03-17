use serenity::{
    all::{Color, CommandInteraction, ComponentInteraction, CreateEmbed},
    client::Context,
};
use tracing::{error, warn};

use crate::utils::response::{
    respond_to_button, respond_to_error_button, respond_to_followup,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer skip command: {}", err);
        return;
    }

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        // Attempt to skip the currently playing song
        let skip_result = match handler.queue().current() {
            Some(track) => track.stop(),
            None => {
                let embed = CreateEmbed::new()
                    .description("There is no song currently playing!")
                    .color(Color::DARK_RED);
                respond_to_followup(command, &ctx.http, embed, false).await;

                return;
            }
        };

        match skip_result {
            // The song was successfully skipped. Notify the channel if the
            // queue is now empty
            Ok(_) => {
                let embed = CreateEmbed::new()
                    .description("Song **skipped!**")
                    .color(Color::DARK_GREEN);
                respond_to_followup(command, &ctx.http, embed, false).await;
            }
            Err(why) => {
                error!("Error skipping track in guild {}: {}", guild_id, why);

                let embed = CreateEmbed::new()
                    .description("Error skipping song!")
                    .color(Color::DARK_RED);
                respond_to_followup(command, &ctx.http, embed, false).await;
            }
        };
    } else {
        warn!("Attempted to skip song but bot is not in voice channel (guild {})", guild_id);
        let embed = CreateEmbed::new()
            .description("Error skipping song! Ensure Poor Jimmy is in a voice channel with **/join**")
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

        // Attempt to skip the currently playing song
        let skip_result = match handler.queue().current() {
            Some(track) => track.stop(),
            None => {
                respond_to_button(
                    command,
                    &ctx.http,
                    format!("There is no song currently playing!"),
                    false,
                )
                .await;

                return;
            }
        };

        match skip_result {
            // The song was successfully skipped. Notify the channel if the
            // queue is now empty
            Ok(_) => {
                respond_to_button(command, &ctx.http, format!("Song **skipped!**"), false).await;
            }
            Err(why) => {
                error!("Error skipping track via button in guild {}: {}", guild_id, why);

                respond_to_error_button(command, &ctx.http, format!("Error skipping song!")).await;
            }
        };
    } else {
        warn!("Attempted to skip song via button but bot is not in voice channel (guild {})", guild_id);
        respond_to_error_button(
            command,
            &ctx.http,
            format!("Error skipping song! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("skip").description("Skip the currently playing song")
}
