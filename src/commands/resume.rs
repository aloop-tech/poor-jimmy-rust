use serenity::{
    all::{Color, CommandInteraction, ComponentInteraction, CreateEmbed},
    client::Context,
};
use songbird::tracks::PlayMode;
use tracing::error;

use crate::utils::response::{respond_to_button, respond_to_error_button, respond_to_followup};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer resume command: {}", err);
        return;
    }

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        let current_song = handler.queue().current();

        // Attempt to grab the current play state of the current song
        let song_state = match &current_song {
            Some(track) => match track.get_info().await {
                Ok(state) => state.playing,
                Err(why) => {
                    println!("Error getting song state: {why}");

                    let embed = CreateEmbed::new()
                        .description("Error resuming song!")
                        .color(Color::DARK_RED);
                    respond_to_followup(command, &ctx.http, embed, false).await;

                    return;
                }
            },
            None => {
                let embed = CreateEmbed::new()
                    .description("There is no song to resume!")
                    .color(Color::DARK_GREEN);
                respond_to_followup(command, &ctx.http, embed, false).await;

                return;
            }
        };

        // If the song is paused, resume it
        match song_state {
            PlayMode::Pause => match current_song {
                Some(song) => match song.play() {
                    Ok(_) => {
                        let embed = CreateEmbed::new()
                            .description("Song **resumed!**")
                            .color(Color::DARK_GREEN);
                        respond_to_followup(command, &ctx.http, embed, false).await;
                    }
                    Err(why) => {
                        println!("Error resuming song: {why}");

                        let embed = CreateEmbed::new()
                            .description("Error resuming song!")
                            .color(Color::DARK_RED);
                        respond_to_followup(command, &ctx.http, embed, false).await;
                    }
                },
                None => {
                    let embed = CreateEmbed::new()
                        .description("There is nothing to resume!")
                        .color(Color::DARK_GREEN);
                    respond_to_followup(command, &ctx.http, embed, false).await;
                }
            },
            _ => {
                let embed = CreateEmbed::new()
                    .description("The song is currently playing!")
                    .color(Color::DARK_GREEN);
                respond_to_followup(command, &ctx.http, embed, false).await;
            }
        };
    } else {
        let embed = CreateEmbed::new()
            .description(
                "Error resuming song! Ensure Poor Jimmy is in a voice channel with **/join**",
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

        let current_song = handler.queue().current();

        // Attempt to grab the current play state of the current song
        let song_state = match &current_song {
            Some(track) => match track.get_info().await {
                Ok(state) => state.playing,
                Err(why) => {
                    println!("Error getting song state: {why}");

                    respond_to_error_button(command, &ctx.http, format!("Error resuming song!"))
                        .await;

                    return;
                }
            },
            None => {
                respond_to_button(
                    command,
                    &ctx.http,
                    format!("There is no song to resume!"),
                    false,
                )
                .await;

                return;
            }
        };

        // If the song is paused, resume it
        match song_state {
            PlayMode::Pause => match current_song {
                Some(song) => match song.play() {
                    Ok(_) => {
                        respond_to_button(command, &ctx.http, format!("Song **resumed!**"), false)
                            .await;
                    }
                    Err(why) => {
                        println!("Error resuming song: {why}");

                        respond_to_error_button(
                            command,
                            &ctx.http,
                            format!("Error resuming song!"),
                        )
                        .await;
                    }
                },
                None => {
                    respond_to_button(
                        command,
                        &ctx.http,
                        format!("There is nothing to resume!"),
                        false,
                    )
                    .await;
                }
            },
            _ => {
                respond_to_button(
                    command,
                    &ctx.http,
                    format!("The song is currently playing!"),
                    false,
                )
                .await;
            }
        };
    } else {
        respond_to_error_button(
            command,
            &ctx.http,
            format!("Error resuming song! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("resume").description("Resume the currently paused song")
}
