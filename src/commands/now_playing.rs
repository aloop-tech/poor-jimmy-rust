use serenity::{
    all::{Color, CommandInteraction, CreateEmbed},
    client::Context,
};
use tracing::{error, warn};

use crate::utils::{
    format::create_progress_bar, response::respond_to_followup, track_utils::TrackMetadata,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer now playing command: {}", err);
        return;
    }

    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        let current_track = match handler.queue().current() {
            Some(track) => track,
            None => {
                warn!("No track currently playing in guild {}", guild_id);
                let embed = CreateEmbed::new()
                    .description("No song is currently playing!")
                    .color(Color::DARK_GREEN);
                respond_to_followup(command, &ctx.http, embed, false).await;
                return;
            }
        };

        // Get track metadata
        let metadata = current_track.data::<TrackMetadata>();
        let title = &metadata.title;

        // Get playback info
        let track_info = match current_track.get_info().await {
            Ok(info) => info,
            Err(err) => {
                error!("Failed to get track info in guild {}: {}", guild_id, err);
                let embed = CreateEmbed::new()
                    .description("Error getting track information!")
                    .color(Color::DARK_RED);
                respond_to_followup(command, &ctx.http, embed, false).await;
                return;
            }
        };

        // Format response with progress bar
        let progress_bar = create_progress_bar(track_info.position, metadata.duration, 20);

        let mut embed = CreateEmbed::new()
            .description(format!("**Now Playing:**\n{}\n\n{}", title, progress_bar))
            .color(Color::DARK_GREEN);

        if let Some(url) = &metadata.thumbnail_url {
            embed = embed.thumbnail(url);
        }

        respond_to_followup(command, &ctx.http, embed, true).await;
    } else {
        warn!(
            "Attempted to get now playing but bot is not in voice channel (guild {})",
            guild_id
        );
        let embed = CreateEmbed::new()
            .description("Error! Ensure Poor Jimmy is in a voice channel with **/join**")
            .color(Color::DARK_RED);
        respond_to_followup(command, &ctx.http, embed, false).await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("now-playing")
        .description("Show the currently playing song with progress")
}
