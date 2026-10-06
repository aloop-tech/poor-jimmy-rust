use poise::CreateReply;
use serde::Deserialize;
use serenity::{
    all::{ButtonStyle, ComponentInteraction, Context as SerenityContext, GuildId},
    builder::{CreateActionRow, CreateButton, CreateEmbed, EditInteractionResponse},
    model::colour::Color,
};
use songbird::input::{Input, YoutubeDl};
use tracing::{debug, error};

use crate::{
    data::{Context, Data, Error},
    utils::{
        response::{error_embed, ok_embed, reply, respond_to_followup_component},
        track_utils::{enqueue, load_metadata, voice_target},
    },
};

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    id: String,
    title: String,
    duration: Option<f64>,
    #[serde(default)]
    thumbnails: Vec<Thumbnail>,
}

/// Search YouTube and choose a video's audio to play
#[poise::command(slash_command, guild_only)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Search query"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    debug!("Searching YouTube for: {}", query);

    // Run yt-dlp to search YouTube
    let output = match tokio::process::Command::new("yt-dlp")
        .args([
            "--default-search",
            "ytsearch5",
            "--dump-json",
            "--no-playlist",
            "--flat-playlist",
            // End of options, so a query starting with "-" isn't parsed as a flag
            "--",
            &query,
        ])
        .output()
        .await
    {
        Ok(output) => output,
        Err(err) => {
            error!("Failed to execute yt-dlp: {}", err);
            return reply(
                ctx,
                error_embed("Failed to search YouTube. Please try again later."),
            )
            .await;
        }
    };

    if !output.status.success() {
        error!(
            "yt-dlp command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return reply(
            ctx,
            error_embed("Failed to search YouTube. Please try again later."),
        )
        .await;
    }

    // Parse the JSON output - yt-dlp returns one JSON object per line
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("Stdout from yt-dlp query: {}", stdout);

    let results: Vec<SearchResult> = stdout
        .lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }
            match serde_json::from_str::<SearchResult>(line) {
                Ok(result) => Some(result),
                Err(err) => {
                    error!("Failed to parse search result: {}", err);
                    error!("Line that failed: {}", line);
                    None
                }
            }
        })
        .take(5)
        .collect();

    debug!("Parsed {} results from query", results.len());

    if results.is_empty() {
        return reply(
            ctx,
            error_embed(format!("No results found for \"{}\"", query)),
        )
        .await;
    }

    let mut response = CreateReply::default();

    // Create an embed for each search result
    for (idx, result) in results.iter().enumerate() {
        let duration_str = result
            .duration
            .map(|d| {
                let total_seconds = d as u64;
                let minutes = total_seconds / 60;
                let seconds = total_seconds % 60;
                format!("{}:{:02}", minutes, seconds)
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let mut embed = CreateEmbed::default()
            .title(format!("{}. {}", idx + 1, result.title))
            .description(format!("Duration: {}", duration_str))
            .url(format!("https://www.youtube.com/watch?v={}", result.id))
            .color(Color::BLUE);

        // Add thumbnail if available (use the last one which is usually highest quality)
        if let Some(thumbnail) = result.thumbnails.last() {
            embed = embed.thumbnail(&thumbnail.url);
        }

        response = response.embed(embed);
    }

    // Create buttons for each result
    let buttons: Vec<CreateButton> = results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            CreateButton::new(format!("search_play_{}", result.id))
                .label(format!("Option {}", idx + 1))
                .style(ButtonStyle::Primary)
        })
        .collect();

    // Discord allows up to 5 buttons per action row, we have max 5 results
    let action_rows: Vec<CreateActionRow> = buttons
        .chunks(5)
        .map(|chunk| CreateActionRow::Buttons(chunk.to_vec()))
        .collect();

    ctx.send(response.components(action_rows)).await?;

    Ok(())
}

pub async fn handle_component(
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
    data: &Data,
    guild_id: GuildId,
) {
    if let Err(err) = interaction.defer(&ctx.http).await {
        error!("Failed to defer search component interaction: {}", err);
        return;
    }

    // Extract video ID from button custom_id (format: "search_play_{video_id}")
    let video_id = match interaction.data.custom_id.strip_prefix("search_play_") {
        Some(id) => id.to_string(),
        None => {
            error!("Invalid custom_id format: {}", interaction.data.custom_id);
            // Delete the search results message
            if let Err(err) = interaction.delete_response(&ctx.http).await {
                error!("Failed to delete search results message: {}", err);
            }
            respond_to_followup_component(
                interaction,
                &ctx.http,
                error_embed("Invalid selection!"),
            )
            .await;
            return;
        }
    };

    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
    debug!("Playing selected video: {}", video_url);

    // Update the message to show we're processing the selection
    let loading_embed = CreateEmbed::default()
        .description("Adding track to queue...")
        .color(Color::BLUE);

    if let Err(err) = interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .embeds(vec![loading_embed])
                .components(vec![]), // Remove buttons
        )
        .await
    {
        error!("Failed to update search results message: {}", err);
    }

    let source: Input = YoutubeDl::new(data.http_client.clone(), video_url).into();

    // Delete the loading message before enqueueing
    if let Err(err) = interaction.delete_response(&ctx.http).await {
        error!("Failed to delete loading message: {}", err);
    }

    let embed = match queue_selection(ctx, data, guild_id, interaction, source).await {
        Ok(title) => ok_embed(format!("**Queued** {}!", title)),
        Err(message) => error_embed(message),
    };
    respond_to_followup_component(interaction, &ctx.http, embed).await;
}

/// Load the chosen video and queue it, joining the user's voice channel if needed.
/// Returns the queued title.
async fn queue_selection(
    ctx: &SerenityContext,
    data: &Data,
    guild_id: GuildId,
    interaction: &ComponentInteraction,
    mut source: Input,
) -> Result<String, String> {
    let target = voice_target(ctx, guild_id, interaction.user.id).await?;
    let metadata = load_metadata(&mut source).await?;
    let title = metadata.title.clone();

    enqueue(ctx, data, guild_id, target, vec![(source, metadata)]).await?;
    Ok(title)
}
