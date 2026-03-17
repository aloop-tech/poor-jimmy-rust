use serenity::{all::CommandInteraction, client::Context};
use tracing::{error, info};

use crate::utils::response::{respond_to_command, respond_to_error};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    match manager.remove(guild_id).await {
        Ok(_) => {
            info!("Successfully left voice channel in guild {}", guild_id);
            respond_to_command(
                command,
                &ctx.http,
                format!("Poor Jimmy **left** the voice channel!"),
                false,
            )
            .await;
        }
        Err(err) => {
            error!(
                "Failed to leave voice channel in guild {}: {}",
                guild_id, err
            );
            respond_to_error(command, &ctx.http, format!("Error leaving voice channel! Ensure Poor Jimmy is in a voice channel with **/join**")).await;
        }
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("leave")
        .description("Remove Poor Jimmy from the voice channel")
}
