mod commands;
mod components;
mod data;
mod handlers;
mod utils;

use std::env;

use data::Data;
use handlers::bot_event;
use reqwest::Client as HttpClient;
use serenity::{
    all::{ActivityData, OnlineStatus},
    client::ClientBuilder,
    prelude::*,
};
use songbird::SerenityInit;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber with environment filter
    // Set RUST_LOG env variable to control log level (e.g., RUST_LOG=debug)
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("poor_jimmy=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Poor Jimmy Discord Bot...");

    // DISCORD_TOKEN is required. Bot will not work without it.
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("DISCORD_TOKEN environment variable not set!");
            std::process::exit(1);
        }
    };

    // Slash commands and buttons arrive regardless of intents. Non-privileged
    // covers GUILDS and GUILD_VOICE_STATES, which /join needs to find the user's channel.
    let intents = GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            pre_command: |ctx| Box::pin(bot_event::pre_command(ctx)),
            event_handler: |ctx, event, _framework, data| {
                Box::pin(bot_event::handle_event(ctx, event, data))
            },
            on_error: |error| Box::pin(bot_event::on_error(error)),
            ..Default::default()
        })
        // Runs once, on the first Ready event, so commands aren't re-registered on reconnects
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                info!("{} is connected! (ID: {})", ready.user.name, ready.user.id);

                let commands = &framework.options().commands;
                info!("Registering {} slash commands globally...", commands.len());

                // Log and carry on if registration fails so the already-registered
                // commands keep working
                match poise::builtins::register_globally(ctx, commands).await {
                    Ok(()) => info!("Successfully registered {} slash commands", commands.len()),
                    Err(err) => error!("Failed to register slash commands: {}", err),
                }

                ctx.set_presence(Some(ActivityData::listening("/play")), OnlineStatus::Online);
                info!("Bot is ready and listening for commands!");

                Ok(Data {
                    http_client: HttpClient::new(),
                    disconnect_timers: Default::default(),
                })
            })
        })
        .build();

    info!("Building Discord client with required intents...");

    let mut client = match ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await
    {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to create Discord client: {}", err);
            std::process::exit(1);
        }
    };

    info!("Starting Discord client connection...");

    if let Err(why) = client.start().await {
        error!("Client error: {}", why);
        std::process::exit(1);
    }
}
