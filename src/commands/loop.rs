use serenity::{
    all::{Color, CommandInteraction, ComponentInteraction, CreateEmbed},
    client::Context,
};
use songbird::tracks::LoopState;
use tracing::error;

use crate::utils::response::{respond_to_button, respond_to_error_button, respond_to_followup};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer loop command: {}", err);
        return;
    }

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        // Grab the currrently playing song
        let current_song = handler.queue().current();

        // Grab the state of the current song
        let is_looping = match &current_song {
            Some(track) => {
                match track.get_info().await {
                    Ok(state) => state.loops.eq(&LoopState::Infinite),
                    // If we can't get the track's state, return early
                    Err(why) => {
                        println!("Error getting song state: {why}");

                        let embed = CreateEmbed::new()
                            .description("Error looping song!")
                            .color(Color::DARK_RED);
                        respond_to_followup(command, &ctx.http, embed, false).await;

                        return;
                    }
                }
            }
            // If the queue is empty, return early
            None => {
                let embed = CreateEmbed::new()
                    .description("There is no song to loop!")
                    .color(Color::DARK_GREEN);
                respond_to_followup(command, &ctx.http, embed, false).await;

                return;
            }
        };

        if is_looping {
            match current_song.unwrap().disable_loop() {
                Ok(_) => {
                    let embed = CreateEmbed::new()
                        .description("Disabled **looping!**")
                        .color(Color::DARK_GREEN);
                    respond_to_followup(command, &ctx.http, embed, false).await;
                }
                // Error disabling loop, return early
                Err(why) => {
                    println!("Error disabling looping: {why}");

                    let embed = CreateEmbed::new()
                        .description("Error looping song!")
                        .color(Color::DARK_RED);
                    respond_to_followup(command, &ctx.http, embed, false).await;
                }
            }
        } else {
            match current_song.unwrap().enable_loop() {
                Ok(_) => {
                    let embed = CreateEmbed::new()
                        .description("Enabled **looping!** Use **/loop** again to disable or **/skip** to skip")
                        .color(Color::DARK_GREEN);
                    respond_to_followup(command, &ctx.http, embed, false).await;
                }
                // Error enabling loop, return early
                Err(why) => {
                    println!("Error looping song: {why}");

                    let embed = CreateEmbed::new()
                        .description("Error looping song!")
                        .color(Color::DARK_RED);
                    respond_to_followup(command, &ctx.http, embed, false).await;
                }
            }
        }
    } else {
        let embed = CreateEmbed::new()
            .description(
                "Error looping song! Ensure Poor Jimmy is in a voice channel with **/join**",
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

        // Grab the currrently playing song
        let current_song = handler.queue().current();

        // Grab the state of the current song
        let is_looping = match &current_song {
            Some(track) => {
                match track.get_info().await {
                    Ok(state) => state.loops.eq(&LoopState::Infinite),
                    // If we can't get the track's state, return early
                    Err(why) => {
                        println!("Error getting song state: {why}");

                        respond_to_error_button(command, &ctx.http, format!("Error looping song!"))
                            .await;

                        return;
                    }
                }
            }
            // If the queue is empty, return early
            None => {
                respond_to_button(
                    command,
                    &ctx.http,
                    format!("There is no song to loop!"),
                    false,
                )
                .await;

                return;
            }
        };

        if is_looping {
            match current_song.unwrap().disable_loop() {
                Ok(_) => {
                    respond_to_button(command, &ctx.http, format!("Disabled **looping!**"), false)
                        .await;
                }
                // Error disabling loop, return early
                Err(why) => {
                    println!("Error disabling looping: {why}");

                    respond_to_error_button(command, &ctx.http, format!("Error looping song!"))
                        .await;
                }
            }
        } else {
            match current_song.unwrap().enable_loop() {
                Ok(_) => {
                    respond_to_button(command, &ctx.http, format!("Enabled **looping!** Use **/loop** again to disable or **/skip** to skip"), false).await;
                }
                // Error enabling loop, return early
                Err(why) => {
                    println!("Error looping song: {why}");

                    respond_to_error_button(command, &ctx.http, format!("Error looping song!"))
                        .await;
                }
            }
        }
    } else {
        respond_to_error_button(
            command,
            &ctx.http,
            format!("Error looping song! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("loop")
        .description("Enable/disable looping for the current song")
}
