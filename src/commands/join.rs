use serenity::{
    all::{Color, CommandInteraction, CreateEmbed, CreateInteractionResponseFollowup},
    client::Context,
};
use songbird::{Event, TrackEvent};
use tracing::{error, info, warn};

use crate::commands::help::get_help_text;
use crate::handlers::track_end::TrackEndNotifier;
use crate::utils::response::respond_to_followup;

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer join command: {}", err);
        return;
    }

    let guild_id = command.guild_id.unwrap();
    let user_id = {
        let member = command.member.as_ref().unwrap();
        member.user.id
    };

    // Extract voice channel ID from cache, ensuring guild reference is dropped
    let voice_channel_id = {
        ctx.cache.guild(guild_id).and_then(|g| {
            g.voice_states
                .get(&user_id)
                .and_then(|voice_state| voice_state.channel_id)
        })
    };

    // Check if we successfully got the guild from cache
    if voice_channel_id.is_none() && ctx.cache.guild(guild_id).is_none() {
        error!("Failed to find guild {} in cache", guild_id);
        let embed = CreateEmbed::new()
            .description("Error joining voice channel")
            .color(Color::DARK_RED);
        respond_to_followup(command, &ctx.http, embed, false).await;
        return;
    }

    let connect_to = match voice_channel_id {
        Some(channel) => channel,
        None => {
            warn!(
                "User {} attempted to use /join but is not in a voice channel (guild {})",
                user_id, guild_id
            );
            let embed = CreateEmbed::new()
                .description("You're not in a voice channel!")
                .color(Color::DARK_RED);
            respond_to_followup(command, &ctx.http, embed, false).await;

            return;
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    info!(
        "Attempting to join voice channel {} in guild {}",
        connect_to, guild_id
    );

    match manager.join(guild_id, connect_to).await {
        Ok(call) => {
            {
                let mut handler = call.lock().await;

                handler.remove_all_global_events();

                handler.add_global_event(
                    Event::Track(TrackEvent::End),
                    TrackEndNotifier {
                        channel_id: command.channel_id,
                        http: ctx.http.clone(),
                        call: call.clone(),
                        guild_id,
                        manager: manager.clone(),
                    },
                );
            } // lock released before any HTTP requests

            info!(
                "Successfully joined voice channel {} in guild {}",
                connect_to, guild_id
            );

            // Send success message
            let success_embed = CreateEmbed::new()
                .description("Poor Jimmy **joined** the voice channel!")
                .color(Color::DARK_GREEN);
            respond_to_followup(command, &ctx.http, success_embed, false).await;

            // Send help message as a second followup
            let help_embed = CreateEmbed::new()
                .description(get_help_text())
                .color(Color::BLUE);

            if let Err(err) = command
                .create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new().embed(help_embed),
                )
                .await
            {
                error!("Failed to send help followup: {}", err);
            }
        }
        Err(err) => {
            error!(
                "Failed to join voice channel {} in guild {}: {}",
                connect_to, guild_id, err
            );
            let embed = CreateEmbed::new()
                .description("Error joining voice channel!")
                .color(Color::DARK_RED);
            respond_to_followup(command, &ctx.http, embed, false).await;
        }
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("join")
        .description("Summon Poor Jimmy to your voice channel")
}
