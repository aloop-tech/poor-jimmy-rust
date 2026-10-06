use tracing::{error, info};

use crate::{
    data::{Context, Error},
    utils::response::{error_embed, ok_embed, reply},
};

/// Updates Jimmy's dependencies. Use cautiously! Only use this when experiencing playback issues!
#[poise::command(slash_command, guild_only, owners_only, rename = "damnit-jimmy")]
pub async fn damnit_jimmy(ctx: Context<'_>) -> Result<(), Error> {
    info!("Received damnit-jimmy command from {}", ctx.author().name);

    // Defer the response since this might take a while
    ctx.defer().await?;

    // Get current version first
    let current_version = get_ytdlp_version().await;

    // Execute the update command using pip for latest version
    let output = match tokio::process::Command::new("pip")
        .args([
            "install",
            "--upgrade",
            "--break-system-packages",
            "yt-dlp[default]",
        ])
        .output()
        .await
    {
        Ok(output) => output,
        Err(err) => {
            error!("Failed to execute yt-dlp update command: {}", err);
            return reply(ctx, error_embed("Failed to update Jimmy's dependencies!")).await;
        }
    };

    // Log the output for debugging
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    info!("yt-dlp update stdout: {}", stdout);
    if !stderr.is_empty() {
        info!("yt-dlp update stderr: {}", stderr);
    }

    // Get new version after update
    let new_version = get_ytdlp_version().await;

    // Send the response based on success or failure
    let result_embed = if output.status.success() {
        let old_ver = current_version.unwrap_or_else(|| "Unknown".to_string());
        let new_ver = new_version.unwrap_or_else(|| "Unknown".to_string());

        let description = if old_ver == new_ver {
            format!(
                "Update completed, but already on latest version.\n\n**Version:** {}",
                new_ver
            )
        } else {
            format!(
                "Successfully updated Jimmy's dependencies!\n\n**Old version:** {}\n**New version:** {}",
                old_ver, new_ver
            )
        };

        ok_embed(description)
    } else {
        error!(
            "yt-dlp update failed with exit code: {:?}",
            output.status.code()
        );

        error_embed("Failed to update Jimmy's dependencies!")
    };

    reply(ctx, result_embed).await
}

async fn get_ytdlp_version() -> Option<String> {
    let output = tokio::process::Command::new("yt-dlp")
        .arg("--version")
        .output()
        .await
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
