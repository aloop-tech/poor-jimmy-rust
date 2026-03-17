use serenity::{all::CommandInteraction, client::Context};
use tracing::info;

use crate::utils::response::respond_to_command;

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let guild_id = command
        .guild_id
        .expect("No Guild ID found on interaction")
        .to_string();

    info!("Ping! From guild id: {guild_id}");

    respond_to_command(command, &ctx.http, format!("Pong!"), false).await;
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("ping").description("Respond with Pong!")
}
