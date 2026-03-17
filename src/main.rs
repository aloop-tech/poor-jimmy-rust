mod commands;
mod components;
mod handlers;
mod utils;

use dotenv::dotenv;
use std::env;

use handlers::bot_event::BotEventHandler;
use reqwest::Client as HttpClient;
use serenity::client::ClientBuilder;
use serenity::prelude::*;
use songbird::SerenityInit;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utils::type_map::HttpKey;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Initialize tracing subscriber with environment filter
    // Set RUST_LOG env variable to control log level (e.g., RUST_LOG=debug)
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("poor_jimmy=info")),
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

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES;

    info!("Building Discord client with required intents...");

    let mut client = match ClientBuilder::new(token, intents)
        .event_handler(BotEventHandler)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
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
