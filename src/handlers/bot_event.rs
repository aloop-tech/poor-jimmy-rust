use serenity::all::{Command, Interaction, Ready};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::gateway::ActivityData;
use serenity::model::user::OnlineStatus;
use tracing::{debug, error, info};

use crate::commands;
use crate::utils::response::{respond_to_error, respond_to_error_button};

/// The primary handler for the bot that handles all
/// the events for the client
pub struct BotEventHandler;

#[async_trait]
impl EventHandler for BotEventHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.as_str();
            let user = &command.user;

            // Every command needs a guild, so reject DMs here instead of in each command
            let Some(guild_id) = command.guild_id else {
                debug!("Rejected command '{}' from user {} in DM", command_name, user.name);
                respond_to_error(&command, &ctx.http, "Poor Jimmy only works in servers!".to_string())
                    .await;
                return;
            };

            debug!(
                "Received command '{}' from user {} in guild {}",
                command_name, user.name, guild_id
            );

            match command_name {
                "clear" => commands::clear::run(&ctx, &command).await,
                "damnit-jimmy" => commands::damnit_jimmy::run(&ctx, &command).await,
                "help" => commands::help::run(&ctx, &command).await,
                "join" => commands::join::run(&ctx, &command).await,
                "leave" => commands::leave::run(&ctx, &command).await,
                "list" => commands::list::run(&ctx, &command).await,
                "loop" => commands::r#loop::run(&ctx, &command).await,
                "now-playing" => commands::now_playing::run(&ctx, &command).await,
                "pause" => commands::pause::run(&ctx, &command).await,
                "ping" => commands::ping::run(&ctx, &command).await,
                "play-title" => commands::play_title::run(&ctx, &command).await,
                "play-url" => commands::play_url::run(&ctx, &command).await,
                "search" => commands::search::run(&ctx, &command).await,
                "skip" => commands::skip::run(&ctx, &command).await,
                "resume" => commands::resume::run(&ctx, &command).await,
                _ => {
                    error!("Unknown command received: {}", command_name);
                    respond_to_error(&command, &ctx.http, format!("Unknown command!")).await;
                }
            };
        } else if let Interaction::Component(command) = interaction {
            let button_id = command.data.custom_id.as_str();
            let user = &command.user;

            if command.guild_id.is_none() {
                debug!("Rejected button '{}' from user {} in DM", button_id, user.name);
                respond_to_error_button(
                    &command,
                    &ctx.http,
                    "Poor Jimmy only works in servers!".to_string(),
                )
                .await;
                return;
            }

            debug!(
                "Received button interaction '{}' from user {}",
                button_id, user.name
            );

            if button_id.starts_with("search_play_") {
                commands::search::handle_component(&ctx, &command).await;
            } else {
                match button_id {
                    "clear" => commands::clear::handle_button(&ctx, &command).await,
                    "loop" => commands::r#loop::handle_button(&ctx, &command).await,
                    "pause" => commands::pause::handle_button(&ctx, &command).await,
                    "resume" => commands::resume::handle_button(&ctx, &command).await,
                    "skip" => commands::skip::handle_button(&ctx, &command).await,
                    _ => {
                        error!("Unknown button interaction received: {}", button_id);
                        respond_to_error_button(&command, &ctx.http, format!("Unknown command!")).await;
                    }
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected! (ID: {})", ready.user.name, ready.user.id);

        let commands = vec![
            commands::clear::register(),
            commands::damnit_jimmy::register(),
            commands::help::register(),
            commands::join::register(),
            commands::leave::register(),
            commands::list::register(),
            commands::r#loop::register(),
            commands::now_playing::register(),
            commands::pause::register(),
            commands::ping::register(),
            commands::play_title::register(),
            commands::play_url::register(),
            commands::resume::register(),
            commands::search::register(),
            commands::skip::register(),
        ];

        info!("Registering {} slash commands globally...", commands.len());

        match Command::set_global_commands(&ctx.http, commands).await {
            Ok(registered_commands) => {
                info!(
                    "Successfully registered {} slash commands",
                    registered_commands.len()
                );
            }
            Err(err) => {
                error!("Failed to register slash commands: {}", err);
            }
        }

        ctx.set_presence(Some(ActivityData::listening("/play")), OnlineStatus::Online);
        info!("Bot is ready and listening for commands!");
    }
}
